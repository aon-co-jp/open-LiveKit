# PORTING (open-LiveKit)

## 現状(2026-09-27)

設計ドキュメント一式に加え、**実装フェーズ1が完了**。`cargo init`で
Rustバイナリプロジェクトを作成し、[`src/schema.rs`](src/schema.rs)に
GraphQLスキーマ(SDL文字列、型のみ)、[`src/resolvers.rs`](src/resolvers.rs)に
Mutation一部の最小ダミー実装を作成。`open-runo-federation`
(`RPoem`のFederation合成エンジン)の`parse_service_sdl`でこのSDLを実際に
読み込み・型抽出できることをビルド・テスト・実行で確認済み
(`cargo run`で6型認識、`cargo test`で3テスト全通過)。詳細は下記
「7. 実装フェーズ1(完了)」を参照。

## 経緯

[`open-tv-chat`](https://github.com/aon-co-jp/open-tv-chat)の通話リレー
サーバー(既定モードで相手にIPを開示しない中継、詳細は
`open-tv-chat/PORTING.md`「5. アーキテクチャ方針の変遷」参照)に使うSFU技術の
選定において、既存OSS(LiveKit/mediasoup/Janus/Jitsi Videobridge)と自前実装の
トレードオフを比較検討した結果、ユーザーから「LiveKitのアーキテクチャを参考に、
新規リポジトリ`open-LiveKit`としてRust + RPoemで(コード流用せず)一から
設計・実装したい。じっくり時間をかけて検索・調査した上で進めてほしい」との
指示を受け、本リポジトリを新設した。

## 1〜5の技術選定 決定事項(2026-09-26)

ユーザーから「1〜5の順で検討、それ以外はAIの判断で」との指示を受け、GitHub
調査(スター数・活動状況・実運用実績・ライセンス)を行った上で以下を決定した。
比較表・詳細根拠は[`README.md`](README.md)「技術選定の決定事項」を参照。

1. **WebRTCスタック**: **webrtc-rs**を採用(5.2k star、Recall.ai等の商用実績、
   1.0に向け安定化中、W3C API準拠95%+)。str0m(627 star、Sans I/O設計)は
   将来のパフォーマンス最適化時の代替候補として保留。
2. **水平スケーリング設計**: **`aruaru-db` + PostgreSQLのDUAL DB(VPS高速
   キャッシュ層)+
   [`aruaru-db-archive`](https://github.com/aon-co-jp/aruaru-db-archive)
   (非公開・Git-on-SQL経由の自動バックアップ層)の2層構成**を採用
   (2026-09-26ユーザー指示で確定: 世界中からの大量アクセス時にVPSの
   ストレージが溢れないよう、古いデータは自動的にGitHubの専用アーカイブ
   リポジトリへ退避し、VPS側は直近データのみ短いTTLで保持する。利用者端末
   側にもログを任意保存できる選択肢を用意)。Redisは新規依存として追加しない。
   リアルタイム用途での性能検証・同期頻度の詳細設計は実装着手時に`aruaru-db`
   側と協調して行う。**age-outポリシー(2026-09-26ユーザー指示)**: VPSの
   HDD空き容量に応じてTTLを6時間→3時間→1時間→30分→5分と段階的に短縮する
   (2026-09-27追記: 容量枯渇寸前の最危険域として20秒段階を追加、6時間→3時間→
1時間→30分→5分→20秒。具体的な閾値・監視間隔はAI判断で実装時に詳細化)。**利用者端末側の
   保存(2026-09-26ユーザー指示)**: 通話(録音/録画)・字幕チャット履歴とも、
   利用者が希望すれば自分の端末の`aruaru-db`(希望すればPostgreSQLとの
   DUAL DB)に残せるようにする。
3. **シグナリングプロトコル**: **RPoem(GraphQL Subscriptions)**を軸に設計。
   gRPC+Protocol Buffersは新規依存として追加しない。
4. **Simulcast対応**: 初期スコープ外、単一レイヤー転送から開始しフェーズ2で
   対応(音声品質・実装の安定を優先)。
5. **翻訳エンジン統合**: 「Agentフック」方式(参加者の音声トラックを複製配信し、
   Whisper→MADLAD-400→Piperで処理した結果を翻訳済みトラックとして
   再パブリッシュ)を採用。

## 6. RPoem GraphQL Subscriptionsスキーマ設計(2026-09-26、設計フェーズ継続)

LiveKitのシグナリング(gRPC+Protocol Buffers)が扱う概念(ルーム参加・退出、
トラック発行/購読、ルーム状態更新)を、`RPoem`(Cosmo互換GraphQL Federation)
のGraphQL Subscriptions/Mutationsで再設計した初期スキーマ案。

```graphql
type Room {
  id: ID!
  name: String!
  participants: [Participant!]!
  createdAt: DateTime!
}

type Participant {
  id: ID!
  displayName: String!       # 相手に開示する表示名(実名等の自動開示はしない)
  tracks: [Track!]!
  connectionMode: ConnectionMode!  # RELAY(既定) or P2P(利用者同意時のみ)
}

enum ConnectionMode {
  RELAY
  P2P
}

type Track {
  id: ID!
  kind: TrackKind!            # AUDIO / VIDEO / TRANSLATED_AUDIO / CAPTION
  ownerParticipantId: ID!
  sourceLanguage: String      # 音声トラックの原言語(Agentフックが判定/指定)
  targetLanguage: String      # TRANSLATED_AUDIO/CAPTIONの場合の翻訳先言語
}

enum TrackKind {
  AUDIO
  VIDEO
  TRANSLATED_AUDIO
  CAPTION
}

type Mutation {
  joinRoom(roomName: String!, displayName: String!): Participant!
  leaveRoom(participantId: ID!): Boolean!
  publishTrack(participantId: ID!, kind: TrackKind!): Track!
  unpublishTrack(trackId: ID!): Boolean!
  requestP2PMode(roomId: ID!, participantId: ID!): Boolean!   # 相手の同意待ち
  respondP2PModeRequest(roomId: ID!, accept: Boolean!): Boolean!
  requestTranslation(participantId: ID!, targetLanguages: [String!]!): Boolean!
    # 最大10ヶ国語まで(open-tv-chat要件)
}

type Subscription {
  roomStateChanged(roomId: ID!): Room!
  trackPublished(roomId: ID!): Track!
  trackUnpublished(roomId: ID!): ID!
  captionReceived(roomId: ID!, targetLanguage: String!): CaptionEvent!
}

type CaptionEvent {
  participantId: ID!
  targetLanguage: String!
  text: String!
  timestamp: DateTime!
}
```

- `requestTranslation`は最大10ヶ国語まで同時指定可能とする(`open-tv-chat`の
  要件)。Agentフック側がこの数だけ翻訳パイプラインを並列起動する想定。
- `requestP2PMode`/`respondP2PModeRequest`は、既定のRELAYモードから
  P2Pモードへの切替が双方の同意なしに行われないようにするための最小限の
  ハンドシェイクを表現したもの(詳細UIは`open-tv-chat`側で設計)。
- 上記はあくまで初期スキーマ案であり、実装着手時に`RPoem`側の実際の
  Federation構成(サブグラフ分割等)に合わせて調整する。

## 7. 実装フェーズ1(完了・2026-09-27)

前回セッション中断時点の課題だった`cargo`PATH未登録は、
`%USERPROFILE%\.cargo\bin`を前置きすることで解決した
([`reference_rust_toolchain_env.md`]記憶メモのとおり)。

- `cargo init --name open-livekit`でRustバイナリプロジェクトを作成。
- [`Cargo.toml`](Cargo.toml): `open-runo-federation`を`path`依存で追加
  (現状はローカル開発機の兄弟ディレクトリ`F:\RPoem`を直接参照。他クローン
  環境・CIでは動かないため、実装が進んだ段階でgitソース依存へ切り替える
  必要がある、と明記した)。
- [`src/schema.rs`](src/schema.rs): `open-tv-chat/PORTING.md`で設計した
  GraphQLスキーマ(Room/Participant/Track/CaptionEvent/Mutation/
  Subscription)をSDL文字列定数`SDL`として実装。`open_runo_federation::sdl::
  parse_service_sdl`でパースし、6つの型すべてが正しく抽出されることを
  確認するテストを追加。
- [`src/resolvers.rs`](src/resolvers.rs): `joinRoom`・`requestTranslation`の
  最小ダミー実装(実データ接続なし)。`requestTranslation`は`open-tv-chat`の
  要件どおり「最大10ヶ国語まで」の上限チェックのみ実装・テスト済み。
- `cargo build`/`cargo run`/`cargo test`いずれも成功
  (`cargo run`で6型認識、`cargo test`で3テスト全通過)。

**未実施(次フェーズ)**: リゾルバの実配線(RPoemゲートウェイへの実際の登録)、
webrtc-rsでの疎通確認、`aruaru-db`ルーム状態スキーマ、翻訳エンジン統合は
まだ着手していない(下記「次回再開ポイント」参照)。

## 次回再開ポイント

「小規模な基本部分から段階的に実装」の方針(ユーザー指示)に沿い、以下の順で
進める。各ステップ完了後は`cargo test`等で動作確認してから次へ進むこと。

1. **(フェーズ2)`webrtc-rs`での最小疎通確認**: 2プロセス(または2スレッド)間
   でPeerConnectionを確立し、ダミー音声トラックを1本転送するだけの最小
   プロトタイプを`open-livekit`内に追加する。まずは`webrtc-rs`の
   Cargo依存追加とバージョン確認から。
2. **`aruaru-db`側のルーム状態スキーマ実装**: ルームID→ノードIDの最小限の
   テーブル定義(`aruaru-db`+PostgreSQLのDUAL DB構成、詳細は上記
   「2. 水平スケーリング設計」参照)。
3. **翻訳エンジン(Whisper単体)を1言語間だけAgentフックに接続する最小構成**:
   `src/resolvers.rs`の`request_translation`ダミー実装を、実際にWhisperの
   ASR出力(テキスト)を受け取って`CaptionEvent`として返せる最小限の経路に
   拡張する(MT/TTSはまだ繋がなくて良い、まず1段階ずつ)。
4. **リゾルバのRPoemゲートウェイへの実配線**: 上記1〜3が個別に動く状態に
   なったら、`RPoem`側のルーターにこのサブグラフを実際に登録し、
   GraphQL経由で呼び出せるようにする。
5. ~~踏み台の利用アプリ許可リスト機構~~ → **2026-09-27完了**、
   [`src/relay.rs`](src/relay.rs)の`is_app_allowed`として実装済み。
6. ~~Hop1/Hop2ノードの定期ランダムローテーション機構~~ → **2026-09-27完了**、
   [`src/relay.rs`](src/relay.rs)の`NodePool`(`pick`/`rotate`)として
   実装済み。

## 9. 実装フェーズ2(完了・2026-09-27): 許可リスト+ノードローテーション

[`src/relay.rs`](src/relay.rs)に以下を実装、`cargo test`で7テスト全通過
(スキーマ3+リゾルバ2+relay4のうち新規4件)。

- `is_app_allowed(app_id: &str) -> bool`: 許可リスト
  (`open-tv-chat`/`open-english`/`aruaru-tokyo`)にあるapp_idのみ許可。
  リスト外(任意の一般サイトのURL等)は拒否。
- `NodePool`: Hop1/Hop2それぞれのノードプール。
  - `pick(random_unit: f64)`: 呼び出し側が渡す`[0.0, 1.0)`の乱数でプールから
    1ノードを選ぶ(乱数生成自体は実装のスコープ外とし、テスト容易性を優先)。
  - `rotate(new_nodes)`: プール内容を丸ごと入れ替える(ローテーション)。
    通話中セッションは影響を受けない設計(呼び出し時に選んだノードを使い
    続ける)という前提は、実際のセッション管理実装時に検証が必要。

**未実施(次フェーズ)**: 実際の乱数源(暗号論的に安全なもの)の選定、
ローテーション間隔の具体的なスケジューリング(cron的な仕組み)、ノード
プールの配布方式(クライアントがどうプール一覧を取得するか)、
webrtc-rs疎通確認(上記1)以降は未着手のまま。

## 8. 踏み台の用途スコープの検討経緯(2026-09-27、3段階の議論)

ユーザーから「独立したVPNアプリ(`aon-vpn`)も同時開発してはどうか」との
提案があり、次の順で検討・確定した。

1. 独立VPNアプリ案 → 各国のVPN法規制(完全禁止国・強い規制国・イランの
   刑事罰化・義務的ログ保持国等)を理由に見送り。
2. 代替案として「踏み台を`aruaru-vpn`という名前で汎用インターネットアクセス
   にも拡張する」案を検討 → ユーザーから「名前を変えても実体はVPNと同じで
   法的リスクは変わらないのでは」との指摘があり、これも撤回。
3. **最終決定(2026-09-27、さらにユーザーから「設計メモ止まりではなく今すぐ
   同時に着手してほしい」との追加指示)**: 二段階踏み台は**任意の一般
   Webサイト等は対象外のまま、`open-tv-chat`・`open-english`・
   `aruaru-tokyo`等の「aon-co-jpが運営する自社アプリの通信であればどれでも
   経由できる」共通リレー基盤として、最初から設計・実装する**
   (Zoom/Discord等と同じ「アプリ固有の通信を運ぶ中継」という法的扱いを
   維持しつつ、対象アプリの範囲だけを自社アプリ群に広げる形。任意サイトを
   代理する汎用VPN機能は持たない)。許可リスト(アプリID等)で対象アプリを
   管理し、リスト外の宛先への中継は拒否する。

詳細は[`README.md`](README.md)「踏み台の用途スコープ」を参照。独立VPNアプリ・
「aruaru-vpn」という名称での汎用インターネットアクセス拡張構想は撤回済み
(自社アプリ限定の共通リレー基盤とは別物)。

具体的な実装(TV CHAT専用の踏み台、Hop1/Hop2分離の技術詳細)は次回再開
ポイント1〜4(webrtc-rs疎通確認等)の中で行う。
