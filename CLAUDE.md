# 開発方針＆開発環境ルール(open-LiveKit)

作業ドライブは`F:\open-LiveKit`。この節は
[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を
**正本**とし、各プロジェクトへコピーして同期する既存の運用ルール継承方針に
準じる(比較的新しいフレームワークの参照資料一覧・AI駆動開発ツールに関する
所感・確認不要の自動継続/リミット解除後の自動再開・白画面バグ等を見逃さない
検証徹底、等の全リポジトリ共通ルールは、詳細をここに複製せず
`open-raid-z/CLAUDE.md`を参照すること)。

## このリポジトリの役割

[LiveKit](https://github.com/livekit/livekit)(Go+Pion製、Apache-2.0)の
アーキテクチャを参考に、コードを一切流用せず一からRust + `RPoem`で再実装する
WebRTC SFU/リアルタイム通信基盤。[`open-tv-chat`](https://github.com/aon-co-jp/open-tv-chat)の
リレーサーバーとして使う。詳細な構想・調査結果は[`README.md`](README.md)を参照。

## 開発姿勢(ユーザー指示・2026-09-26)

「LiveKitをそっくり真似て...じっくりとゆっくりとGoogle検索とGithub調査で
設計と実装、開発の為の検索と調査もしっかりと行なった上で設計と開発に移って
欲しい」との指示を受けている。以下を徹底すること:

- **コードの複製・流用は禁止**: LiveKit(およびPion等その依存)のソースコードを
  直接コピー・移植しない。参考にするのはアーキテクチャ・設計思想・機能セット
  のみ(`RCosmo`/`RFrontEnd`/`RPoem`と同じ既存方針)。
- **調査を急がない**: 実装の各段階(WebRTCスタック選定・水平スケーリング設計・
  シグナリングプロトコル設計等)で、Google検索・GitHub調査による裏付けを
  取ってから設計・実装に進む。推測や記憶だけで設計を決め打ちしない。
- **一気に実装へ進まない**: 設計ドキュメントを充実させるフェーズを重視し、
  ユーザーとの方針確認を都度挟みながら段階的に進める。

## 参考にする範囲・参考にしない範囲

| 参考にする | 参考にしない(自前設計) |
|---|---|
| SFUモデル(選択的転送、デコード/再エンコードなし) | Go実装そのもの・Pionのソースコード |
| ルームベースの参加者管理という概念 | Redis依存を含む具体的な実装コード |
| Simulcast(複数レイヤー配信)という概念 | LiveKitのライセンス表記・商標・ブランド名の使用 |
| 水平スケーリング・マルチリージョンという設計思想 | (実装段階で追記) |

## 技術方針(構想、実装未着手)

- WebRTCスタック: **webrtc-rs**(2026-09-26決定。5.2k star・商用実績・1.0に
  向け安定化中。str0mは将来のパフォーマンス最適化時の代替候補)。
- サーバーサイド: Rust + [`RPoem`](https://github.com/aon-co-jp/RPoem)
  (Cosmo互換、REST API不要、Tomcat互換)。
- 水平スケーリング(ノード状態管理): **`aruaru-db` + PostgreSQLのDUAL DB
  (VPS高速キャッシュ層)+
  [`aruaru-db-archive`](https://github.com/aon-co-jp/aruaru-db-archive)
  (非公開・Git-on-SQL経由の自動バックアップ層)の2層構成**(2026-09-26
  ユーザー指示で決定。VPSストレージの溢れ防止・利用者端末側の任意ログ保存
  も含む。Redisは新規依存として追加しない)。
- シグナリングプロトコル: **RPoemのGraphQL Subscriptions**を軸に設計
  (gRPC+Protocol Buffersは新規依存として追加しない)。
- Simulcast: 初期スコープ外(フェーズ2)。
- 翻訳エンジン統合: 「Agentフック」方式(音声トラック複製→
  Whisper→MADLAD-400→Piper→翻訳済みトラック再パブリッシュ)。
  詳細根拠は[`README.md`](README.md)「技術選定の決定事項」、経緯は
  [`PORTING.md`](PORTING.md)を参照。

## HANDOFF

- **2026-09-26 リポジトリ新設**: `aon-co-jp/open-LiveKit`を新規作成。
  LiveKitのアーキテクチャ調査(GitHub公式リポジトリ・公式ドキュメント・
  公式ブログを調査)を`README.md`にまとめた段階。
- **2026-09-26 技術選定1〜5を決定**: ユーザー指示「1〜5の順で検討、それ以外は
  AIの判断で」を受け、(1)WebRTCスタック=webrtc-rs、(2)水平スケーリング=
  `aruaru-db`+PostgreSQL(2026-09-26ユーザー指示で確定)、(3)シグナリング=
  RPoem GraphQL Subscriptions、(4)Simulcastはフェーズ2見送り、(5)翻訳エンジンは
  Agentフック方式、を決定。
- **2026-09-26 age-outポリシー+利用者端末保存+GraphQLスキーマ初期案**:
  VPSのHDD空き容量に応じてage-out TTLを6時間→3時間→1時間→30分→5分→20秒
  (2026-09-27追記、容量枯渇寸前の最危険域)と段階的に短縮する方針
  (閾値詳細はAI判断)、通話(録音/録画)・字幕チャット履歴を利用者が
  希望すれば端末側`aruaru-db`(+希望時PostgreSQLとのDUAL DB)に残せる方針を
  追加。`RPoem`向けGraphQL Subscriptions/Mutationsの初期スキーマ案
  (Room/Participant/Track/CaptionEvent等)を作成([`PORTING.md`](PORTING.md)
  「6. RPoem GraphQL Subscriptionsスキーマ設計」)。
- **2026-09-27 実装フェーズ1完了**: `cargo`のPATH設定
  (`%USERPROFILE%\.cargo\bin`)を解決し、`cargo init`でRustプロジェクトを
  作成。[`src/schema.rs`](src/schema.rs)にGraphQLスキーマ(SDL文字列)、
  [`src/resolvers.rs`](src/resolvers.rs)に`joinRoom`/`requestTranslation`の
  最小ダミー実装を追加。`open-runo-federation`(`RPoem`)の
  `parse_service_sdl`でこのSDLが実際にパースできることを`cargo build`/
  `cargo run`/`cargo test`で確認済み(3テスト全通過)。`Cargo.toml`の
  `open-runo-federation`依存は現状ローカルpath依存(`F:\RPoem`、他クローン
  環境では動かない、後でgit依存へ切替要)。次は「小規模な基本部分から」の
  方針どおりwebrtc-rs疎通確認へ進む。次回再開時は[`PORTING.md`](PORTING.md)
  「次回再開ポイント」を参照。
