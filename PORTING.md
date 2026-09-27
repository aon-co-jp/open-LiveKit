# PORTING (open-LiveKit)

## 現状(2026-09-26)

リポジトリ新設直後。LiveKitのアーキテクチャ調査(公式GitHubリポジトリ・
公式ドキュメント「LiveKit SFU」・公式ブログ「How we built a globally
distributed mesh network to scale WebRTC」を調査)を`README.md`にまとめた
段階。コード実装は0行。

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

## 次回再開ポイント

設計フェーズは(1)WebRTCスタック、(2)水平スケーリング(DUAL DB+アーカイブ+
age-out)、(3)シグナリングスキーマ初期案、(4)Simulcast方針、(5)翻訳エンジン
統合方式まで一通り出揃った。ユーザー指示により「設計が完成したら小規模な
基本部分から実装フェーズへ」進める。次回は以下の順で着手する想定:

1. 上記6のGraphQLスキーマを`RPoem`上に実際に定義する(型定義のみ、
   リゾルバは最小限のダミー実装)。
2. `webrtc-rs`を使った最小限の疎通確認(2プロセス間でPeerConnectionを
   確立し、ダミー音声トラックを1本転送するだけの最小プロトタイプ)。
3. 上記1・2が繋がったら、`aruaru-db`側のルーム状態スキーマ(ルームID→
   ノードID等の最小限のテーブル)を実装する。
4. 翻訳エンジン(Whisper単体)を1言語間だけAgentフックに接続する最小構成へ
   拡張する。

「小規模な基本部分から」の方針に沿い、上記1→2→3→4の順で段階的に実装し、
都度動作確認してから次のステップへ進める。

## セッション中断メモ(2026-09-26、リミットのため停止)

実装フェーズ1(`cargo init`によるRustプロジェクト雛形作成)に着手しようとした
ところで、この開発機の`cargo`コマンドがPATHに未登録のため実行できなかった
([`reference_rust_toolchain_env.md`]記憶メモのとおり`%USERPROFILE%\.cargo\bin`
の前置きが必要)。コード実装は0行のまま。次回セッション再開時は、まず
`cargo`のPATH設定を確認した上で、上記「次回再開ポイント」の1
(GraphQLスキーマを`RPoem`上に定義)から着手すること。
