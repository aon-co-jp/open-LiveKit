//! [`crate::schema`]のMutation/Subscriptionに対する最小ダミーリゾルバ。
//!
//! 実装フェーズ1の時点では、実際のルーム状態・WebRTCトラック・翻訳エンジンとの
//! 接続は行わない。「型は本物、中身はダミー」の状態で、上位(RPoemのFederation
//! ゲートウェイ)への配線経路が成立することだけを確認する段階。以降のフェーズ
//! (webrtc-rs疎通確認・aruaru-dbルーム状態・翻訳Agentフック)で、対応する
//! ダミー実装を1つずつ実データに置き換えていく。

/// `Mutation.joinRoom`の最小ダミー実装。常に固定の`Participant`風データを返す
/// (実データ型はまだ定義していないため、フィールド名だけを示す文字列を返す)。
pub fn join_room(room_name: &str, display_name: &str) -> DummyParticipant {
    DummyParticipant {
        id: format!("participant-dummy-{display_name}"),
        display_name: display_name.to_string(),
        room_name: room_name.to_string(),
    }
}

/// `Mutation.requestTranslation`の最小ダミー実装。
///
/// `open-tv-chat`の要件どおり、同時翻訳先言語は最大10ヶ国語までとする。
/// この段階ではAgentフック(Whisper/MADLAD-400/Piper)への実際の接続は
/// 行わず、上限チェックのみを行う。
pub fn request_translation(target_languages: &[String]) -> Result<(), &'static str> {
    const MAX_SIMULTANEOUS_TARGET_LANGUAGES: usize = 10;
    if target_languages.is_empty() {
        return Err("targetLanguages must not be empty");
    }
    if target_languages.len() > MAX_SIMULTANEOUS_TARGET_LANGUAGES {
        return Err("targetLanguages exceeds the 10-language simultaneous limit");
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DummyParticipant {
    pub id: String,
    pub display_name: String,
    pub room_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_room_returns_dummy_participant_tied_to_room() {
        let p = join_room("room-1", "alice");
        assert_eq!(p.room_name, "room-1");
        assert_eq!(p.display_name, "alice");
    }

    #[test]
    fn request_translation_rejects_empty_and_over_limit() {
        assert!(request_translation(&[]).is_err());

        let eleven: Vec<String> = (0..11).map(|i| format!("lang-{i}")).collect();
        assert!(request_translation(&eleven).is_err());

        let ten: Vec<String> = (0..10).map(|i| format!("lang-{i}")).collect();
        assert!(request_translation(&ten).is_ok());
    }
}
