# open-LiveKit

[LiveKit](https://github.com/livekit/livekit)(Go + Pion製、Apache-2.0)の
アーキテクチャ・機能セットを参考に、**コードを一切流用せず一から**
Rust + [`RPoem`](https://github.com/aon-co-jp/RPoem)
([WunderGraph Cosmo](https://github.com/wundergraph/cosmo)互換のGraphQL
Federation実装)で再実装するWebRTC SFU(Selective Forwarding Unit)/
リアルタイム通信基盤。[`open-tv-chat`](https://github.com/aon-co-jp/open-tv-chat)の
リレーサーバー(既定モードで相手にIPを開示しない通話中継)として使う。

**現時点は設計ドキュメント段階**であり、実装はまだ着手していない。
`RCosmo`/`RFrontEnd`/`RPoem`と同じ「既存実装のコードを一切流用せず、
互換の自前実装を一から開発する」方針(`aon-co-jp`エコシステム共通)に従う。

## 目的

- [`open-tv-chat`](https://github.com/aon-co-jp/open-tv-chat)の通話を、
  既定モードで運営(aon-co-jp)側サーバー経由のリレーとして中継する
  (相手に自分の端末IPを見せないため)。
- LiveKitの実績あるアーキテクチャ(SFUモデル・ルームベースの参加者管理・
  水平スケーリング設計等)を参考にしつつ、Pion(Go)ではなくRust製の
  WebRTCスタックで組み、`RPoem`/`open-web-server`と同一スタックに統合する。

## LiveKitのアーキテクチャ調査結果(2026-09-26、GitHub/公式ドキュメント調査)

| 項目 | LiveKit(参考元) |
|---|---|
| 実装言語 | Go |
| WebRTCライブラリ | [Pion](https://github.com/pion/webrtc)(Go製WebRTC実装) |
| ライセンス | Apache-2.0 |
| モデル | SFU(参加者が発行したトラックをサーバーが選択的に他の参加者へ転送。
  サーバー側でのデコード/再エンコードは行わない) |
| ルーム管理 | ルーム単位で参加者(人・デバイス・AIエージェント)を管理。参加者が
  トラック発行/メッセージ送信するとルーム状態を即座に更新し、シグナリング
  経由で他参加者に通知 |
| 水平スケーリング | Redisを使ったノード間ルーティング(ノード一覧・ルーム名→
  ノードIDのマッピングをRedis上のハッシュで管理、シグナリングメッセージは
  Redis pub/subで配送、同一ノード内はLocalRouterで直接ディスパッチ) |
| マルチリージョン | セッションを「特定のマシンに固定された物理的な存在」では
  なく「複数サーバー/複数データセンターにまたがりうる論理的な存在」として
  扱う設計(参加者は最寄りのサーバーに接続してレイテンシを最小化) |
| プロトコル | WebRTC(メディア) + gRPC(サーバーAPI、複数言語向けバインディング
  生成用にProtocol Buffersで定義) |
| Simulcast | 720p/360p/180p等の複数レイヤーを並行送信し、SFUが購読者ごとに
  最適なレイヤーを選んで転送 |
| ネットワーク処理 | UDP/TCP/TURNフォールバックに対応 |
| 認証 | JWTベース |
| 付加機能 | AIエージェントのディスパッチ、SIPテレフォニー連携、Webhook、
  エンドツーエンド暗号化 |
| リポジトリ構成 | `cmd/`(コマンド)、`pkg/`(パッケージ本体)、`deploy/`
  (デプロイ設定)、`test/`(テスト) |

参照: [LiveKit公式リポジトリ](https://github.com/livekit/livekit)、
[LiveKit SFU内部ドキュメント](https://docs.livekit.io/reference/internals/livekit-sfu/)、
[LiveKitのスケーリング解説(公式ブログ)](https://blog.livekit.io/scaling-webrtc-with-distributed-mesh/)。

## Rust再実装の方針(構想・実装未着手)

- **モデルはSFUを踏襲**: メディアのデコード/再エンコードを行わず、参加者間の
  トラックを選択的に転送する設計はそのまま踏襲する。
- **WebRTCスタックはRust製を採用予定**: Pion(Go)相当の役割を、Rust製の
  WebRTC実装([webrtc-rs](https://github.com/webrtc-rs/webrtc)や
  [str0m](https://github.com/algesten/str0m)等、実装時に改めて比較検討)で
  代替する。
- **ルーム管理・水平スケーリング設計は参考にしつつ再設計**: Redis依存を
  そのまま踏襲するか、`aruaru-db`等の自社アセットで代替できないか含めて
  実装着手時に検討する(推測で決め打ちしない)。
- **シグナリングプロトコル**: gRPC + Protocol Buffersを参考にしつつ、
  `RPoem`(GraphQL Federation)側との親和性を踏まえた設計を実装着手時に
  詳細化する。
- **`open-tv-chat`との連携**: 翻訳エンジン(Whisper/MADLAD-400/Piper、
  [`open-tv-chat`のPORTING.md](https://github.com/aon-co-jp/open-tv-chat/blob/main/PORTING.md)
  参照)を、参加者が発行した音声トラックにフックする形で統合する構想
  (LiveKitのAgentディスパッチ相当の仕組みを自前実装する想定)。

## 技術選定の決定事項(2026-09-26、調査の上でAI判断により決定)

ユーザーより「1〜5の順で検討、それ以外はAIの判断で」との指示を受け、
GitHub調査に基づき以下の通り決定した。

### 1. WebRTCスタック: webrtc-rs を採用

| 項目 | [webrtc-rs](https://github.com/webrtc-rs/webrtc) | [str0m](https://github.com/algesten/str0m) |
|---|---|---|
| スター数 | 5.2k | 627 |
| ライセンス | MIT OR Apache-2.0 | MIT OR Apache-2.0 |
| 設計 | フルスタックのWebRTC実装(PeerConnection API相当)、W3C API準拠95%+ | Sans I/O(ネットワークI/Oを外部化した状態機械)、スレッドレス・ロックフリー |
| 成熟度 | v0.21、1.0に向けて安定化中 | 発展段階、SFU用途を意識した設計だが本番実績は限定的 |
| 実運用実績 | Recall.ai・Stream Chat・ChannelTalk等がスポンサー(商用利用実績あり) | BitWHIP(CLI WebRTC Agent)等 |

**決定**: 実績・成熟度・コミュニティ規模で優位な**webrtc-rs**を第一候補として
採用する。str0mのSans I/O設計はSFU実装により適した特性を持つため、
webrtc-rs採用後にパフォーマンス上の課題が出た場合の代替候補として記録しておく。

### 2. 水平スケーリング設計: `aruaru-db`(高速キャッシュ)+ 自動アーカイブの2層構成(2026-09-26ユーザー指示で確定)

LiveKitはノード間ルーティング状態(ルーム名→ノードIDのマッピング等)をRedisで
管理しているが、Redisを新規依存として追加せず、**`aruaru-db` + PostgreSQL
のDUAL DB構成(自社製のCockroachDB×Snowflake×Git-on-SQLハイブリッド分散DBと
PostgreSQLを並行運用)を軸に、以下の2層構成**を採用する
(2026-09-26ユーザー指示: 「aruaru-db+PostgreSQLのDUAL DBが基本」)。

- **VPS上の`aruaru-db` + PostgreSQL(DUAL DB・高速キャッシュ層)**: アクティブな
  ルーム/ノードルーティング状態・直近データのみを短いTTLで保持する。世界中
  からの大量アクセス時にVPSのストレージが溢れないよう、古いデータは長期
  保持しない。
- **[`aruaru-db-archive`](https://github.com/aon-co-jp/aruaru-db-archive)
  (非公開・自動バックアップ層)**: データが古くなると、`aruaru-db`の
  Git-on-SQLバージョン管理機能経由で自動的にコミット・pushされるアーカイブ
  先の新規リポジトリ。VPS側は同期後に古いデータを破棄できるため容量が
  溢れない。
- **利用者端末側のオプション保存**: 通話ログ等を、利用者が希望すれば自分の
  端末側にも保存できる選択肢を用意し、VPS側へ長期データを溜め込まない設計を
  基本とする。

`aruaru-db`はPostgreSQLワイヤプロトコル互換を志向する自社DBのため、既存の
PostgreSQLエコシステム(ドライバ・運用知見)を活かせる。リアルタイム用途
(ノードルーティングの低レイテンシな読み書き)に耐えるか、Git-on-SQL経由の
自動同期の具体的な実装(同期頻度・age-out判定条件等)は、実装着手時に
`aruaru-db`側と協調して詳細設計・性能検証する。

### 3. シグナリングプロトコル: RPoem(GraphQL Federation)を軸に設計

LiveKitはgRPC + Protocol Buffersだが、`open-tv-chat`/`open-LiveKit`は
「REST API不要」を掲げる[`RPoem`](https://github.com/aon-co-jp/RPoem)
(Cosmo互換GraphQL Federation)を採用済みのため、シグナリング(ルーム参加・
トラック発行/購読通知等のリアルタイムイベント)は**GraphQL Subscriptionsを
軸に設計する**方針とする。gRPCは新規依存として追加しない。

### 4. Simulcast(複数解像度レイヤー)対応: 初期スコープ外、フェーズ2で対応

初期実装は単一レイヤー(発行された解像度をそのまま転送)のシンプルな構成とし、
帯域適応・複数レイヤー同時送信(Simulcast)は初期スコープに含めない
(実装複雑度が高く、まずSFUの基本機能を成立させることを優先するため)。
音声品質(帯域が細くても音声翻訳が破綻しないこと)を優先し、映像の
Simulcastは実装が安定した後のフェーズ2課題とする。

### 5. 翻訳エンジンとの統合方式: 「Agentフック」方式で自前設計

LiveKitのAgentディスパッチ(音声パイプラインへ外部処理をフックする仕組み)を
参考に、参加者が発行した音声トラックをSFU側でAgent(翻訳パイプライン)へ
複製配信し、Whisper(ASR)→MADLAD-400(MT)→Piper(TTS)の順で処理した結果を
「翻訳済み音声トラック」として同じルームに再パブリッシュする方式を採用する
(詳細は[`open-tv-chat`のPORTING.md](https://github.com/aon-co-jp/open-tv-chat/blob/main/PORTING.md)
「1. 音声翻訳エンジン調査結果」と合わせて実装着手時に詳細設計する)。

## 開発方針

このリポジトリの開発ルールは[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の
`CLAUDE.md`を正本とする、`aon-co-jp`エコシステム共通の運用ルール継承方針に従う。
詳細は[`CLAUDE.md`](CLAUDE.md)を参照。

## 現在の到達点

2026-09-26時点: リポジトリ新設、LiveKitのアーキテクチャ調査(本README)のみ。
実装は未着手。じっくり時間をかけて調査・設計を行った上で実装に移る方針
(ユーザー指示)のため、次回セッション以降もGoogle検索・GitHub調査(WebRTC
スタックの選定、水平スケーリング設計の詳細等)を継続する。次回再開ポイントは
[`PORTING.md`](PORTING.md)を参照。
