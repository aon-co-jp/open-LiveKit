//! webrtc-rsでの最小疎通確認(実装ロードマップ「次回再開ポイント」1)。
//!
//! 同一プロセス内に2つの`RTCPeerConnection`(pc1=送信側/オファラー、
//! pc2=受信側/アンサラー)を立て、ダミーの音声トラックを1本
//! pc1→pc2へ送るところまでを確認する最小プロトタイプ。本番のSFUでは
//! 複数拠点間・複数プロセス間の接続になるが、ここではまず「webrtc-rsで
//! PeerConnectionを確立し、トラックのネゴシエーションが成立する」ことだけを
//! 確認する(小規模な基本部分から段階的に進める方針)。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use tokio::time::{sleep, Instant};
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTPCodecType};
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

/// pc1(送信側)からpc2(受信側)へダミー音声トラックを1本送り、
/// 双方のPeerConnectionが`Connected`になり、かつpc2側で`on_track`が
/// 発火することを確認する。成功時は`Ok(())`を返す。
pub async fn probe_dummy_audio_track_handshake() -> webrtc::error::Result<()> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(Registry::new())
        .build();

    let config = RTCConfiguration::default();
    let pc1 = Arc::new(api.new_peer_connection(config.clone()).await?);
    let pc2 = Arc::new(api.new_peer_connection(config).await?);

    // ICE候補をループバックで相互に転送する(同一プロセス内の疎通確認なので
    // シグナリングサーバーは介さず、直接コールバックで橋渡しする)。
    {
        let pc2_for_candidate = Arc::clone(&pc2);
        pc1.on_ice_candidate(Box::new(move |candidate| {
            let pc2 = Arc::clone(&pc2_for_candidate);
            Box::pin(async move {
                if let Some(candidate) = candidate {
                    if let Ok(init) = candidate.to_json() {
                        let _ = pc2.add_ice_candidate(init).await;
                    }
                }
            })
        }));
    }
    {
        let pc1_for_candidate = Arc::clone(&pc1);
        pc2.on_ice_candidate(Box::new(move |candidate| {
            let pc1 = Arc::clone(&pc1_for_candidate);
            Box::pin(async move {
                if let Some(candidate) = candidate {
                    if let Ok(init) = candidate.to_json() {
                        let _ = pc1.add_ice_candidate(init).await;
                    }
                }
            })
        }));
    }

    // 状態フラグをコールバック側でセットし、呼び出し側は短間隔でポーリング
    // する(`Notify`は「待ち始める前にイベントが発火してしまう」競合が
    // 起きうるため、見逃しの無いフラグ方式にしている)。
    let pc1_connected = Arc::new(AtomicBool::new(false));
    {
        let flag = Arc::clone(&pc1_connected);
        pc1.on_peer_connection_state_change(Box::new(move |state| {
            if state == RTCPeerConnectionState::Connected {
                flag.store(true, Ordering::SeqCst);
            }
            Box::pin(async {})
        }));
    }
    let pc2_connected = Arc::new(AtomicBool::new(false));
    {
        let flag = Arc::clone(&pc2_connected);
        pc2.on_peer_connection_state_change(Box::new(move |state| {
            if state == RTCPeerConnectionState::Connected {
                flag.store(true, Ordering::SeqCst);
            }
            Box::pin(async {})
        }));
    }

    // pc2がトラックを受信したらフラグを立てる(ダミー音声トラックの転送確認)。
    let track_received = Arc::new(AtomicBool::new(false));
    {
        let flag = Arc::clone(&track_received);
        pc2.on_track(Box::new(move |track, _receiver, _transceiver| {
            let flag = Arc::clone(&flag);
            Box::pin(async move {
                if track.kind() == RTPCodecType::Audio {
                    flag.store(true, Ordering::SeqCst);
                }
            })
        }));
    }

    // pc1にダミー音声トラック(Opus)を1本追加する。
    let dummy_track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_OPUS.to_owned(),
            ..Default::default()
        },
        "dummy-audio".to_owned(),
        "open-livekit-probe".to_owned(),
    ));
    pc1.add_track(Arc::clone(&dummy_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await?;

    // Offer/Answerを交換する(SDPのやり取り自体はシグナリングサーバー経由に
    // なる想定だが、ここでは同一プロセス内なので直接受け渡す)。
    let offer = pc1.create_offer(None).await?;
    pc1.set_local_description(offer.clone()).await?;
    pc2.set_remote_description(offer).await?;

    let answer = pc2.create_answer(None).await?;
    pc2.set_local_description(answer.clone()).await?;
    pc1.set_remote_description(answer).await?;

    // 両者がConnectedになり、pc2がトラックを受信するまで短間隔でポーリング
    // する(タイムアウト付き。CI等の環境でハングし続けないようにする)。
    // webrtc-rsの`on_track`は実際にRTPパケットが届くまで発火しないため、
    // 接続確立後はダミーのOpusサンプルを定期的に書き込み続ける。
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if pc1_connected.load(Ordering::SeqCst) && pc2_connected.load(Ordering::SeqCst) {
            let _ = dummy_track
                .write_sample(&Sample {
                    data: Bytes::from_static(&[0u8; 8]),
                    duration: Duration::from_millis(20),
                    ..Default::default()
                })
                .await;
        }
        if track_received.load(Ordering::SeqCst) {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }

    let pc1_state = pc1.connection_state();
    let pc2_state = pc2.connection_state();
    let track_ok = track_received.load(Ordering::SeqCst);

    pc1.close().await?;
    pc2.close().await?;

    if pc1_state != RTCPeerConnectionState::Connected {
        return Err(webrtc::Error::new(format!(
            "pc1 did not reach Connected state (got {pc1_state:?})"
        )));
    }
    if pc2_state != RTCPeerConnectionState::Connected {
        return Err(webrtc::Error::new(format!(
            "pc2 did not reach Connected state (got {pc2_state:?})"
        )));
    }
    if !track_ok {
        return Err(webrtc::Error::new(
            "pc2 never received the dummy audio track (on_track did not fire)".to_owned(),
        ));
    }

    Ok(())
}

/// テスト専用: `RTCSessionDescription`/`RTCIceCandidateInit`を実際に
/// 使っていることを型レベルで示すための最小ヘルパー(未使用importの
/// 警告を避けるためだけの用途で、本実装では上の関数内でのみ使用する)。
#[allow(dead_code)]
fn _type_usage_anchor(_sdp: RTCSessionDescription, _ice: RTCIceCandidateInit) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dummy_audio_track_handshake_succeeds() {
        probe_dummy_audio_track_handshake()
            .await
            .expect("webrtc-rs PeerConnection handshake + dummy track transfer must succeed");
    }
}
