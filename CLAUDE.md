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
二段階踏み台(Hop1/Hop2)として使う。**この踏み台はTV CHATの通話トラフィック
専用とし、一般的なインターネットアクセスの代理はしない**(独立VPNアプリ・
汎用アクセス拡張〈通称「aruaru-vpn」〉のいずれも検討の末に撤回、2026-09-27
最終決定。経緯は[`PORTING.md`](PORTING.md)「8. 踏み台の用途スコープの検討
経緯」参照)。詳細な構想・調査結果は[`README.md`](README.md)を参照。

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
- **2026-09-27 踏み台の用途スコープを3段階の議論を経て最終決定**: ユーザーの
  「独立VPNアプリ(`aon-vpn`)も同時開発してはどうか」提案 → 各国VPN法規制
  (完全禁止国・強い規制国・イランの刑事罰化・義務的ログ保持国等)を理由に
  見送り → 代替案「踏み台を`aruaru-vpn`という名前で汎用アクセスにも拡張」も
  検討したが、ユーザーから「名前を変えても実体はVPNと同じで法的リスクは
  変わらない」との指摘で**この拡張構想も撤回**。**最終的に、二段階踏み台は
  一般インターネットアクセスの代理は一切せず(Zoom/Discord型の「アプリ固有
  中継」に収める)、対象範囲は`open-tv-chat`・`open-english`・`aruaru-tokyo`
  等の「aon-co-jp運営の自社アプリの通信であればどれでも経由できる」共通
  リレー基盤として、設計メモ止まりではなく最初から実装することで確定**
  (ユーザー指示: 「1(TV CHAT専用)+2(将来の自社アプリ拡張)を今すぐ同時に」)。
  許可リスト(アプリID等)で対象アプリを管理し、リスト外の宛先への中継は
  拒否する。さらにユーザー指示で、**Hop1/Hop2ノードを固定にせず定期的に
  ランダムローテーションする**設計を追加(特定国のファイアウォール/DPIに
  よるブロックリスト化対策、Tor Bridge等と同種の検閲回避手法)。詳細は
  [`PORTING.md`](PORTING.md)「8. 踏み台の用途スコープの検討経緯」、
  [`README.md`](README.md)「踏み台の用途スコープ」「Hop1/Hop2ノードの
  定期的なランダムローテーション」を参照。次回再開ポイントは
  webrtc-rs疎通確認等(1〜4)に加え、許可リスト機構(5)・ノードローテーション
  機構(6)を実装する。
- **2026-09-27 実装フェーズ2完了**: [`src/relay.rs`](src/relay.rs)に
  許可リスト(`is_app_allowed`)とHop1/Hop2ノードプール(`NodePool::pick`/
  `rotate`)を実装。`cargo test`で7テスト全通過。実際の乱数源選定・
  ローテーションのスケジューリング・ノードプール配布方式は未着手。
  次はwebrtc-rs疎通確認(次回再開ポイント1)へ進む。

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**
