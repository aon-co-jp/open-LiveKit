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
   側と協調して行う。
3. **シグナリングプロトコル**: **RPoem(GraphQL Subscriptions)**を軸に設計。
   gRPC+Protocol Buffersは新規依存として追加しない。
4. **Simulcast対応**: 初期スコープ外、単一レイヤー転送から開始しフェーズ2で
   対応(音声品質・実装の安定を優先)。
5. **翻訳エンジン統合**: 「Agentフック」方式(参加者の音声トラックを複製配信し、
   Whisper→MADLAD-400→Piperで処理した結果を翻訳済みトラックとして
   再パブリッシュ)を採用。

## 次回再開ポイント

技術選定(1〜5)は一通り決定済み。次回は以下を検討:
- `aruaru-db` + PostgreSQLでのノードルーティング状態管理の詳細設計
  (スキーマ・読み書きのレイテンシ要件の整理)
- webrtc-rsを使った最小限のSFUプロトタイプ(2者間・単一レイヤー転送のみ)の
  実装着手時期の判断
- RPoem側のGraphQL Subscriptionsスキーマ設計(ルーム参加・トラック発行/購読
  イベントの型定義)
