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

## 未確定事項(次回セッション以降、じっくり調査しながら詰める)

1. **WebRTCスタックの選定**: [webrtc-rs](https://github.com/webrtc-rs/webrtc)
   (Pion相当のRust実装)と[str0m](https://github.com/algesten/str0m)
   (Sans I/O設計のRust実装)を中心に、本番運用実績・API設計・保守状況を
   Google検索・GitHub調査(Issue/PRの活発さ、実運用事例等)で比較する。
2. **水平スケーリング設計**: LiveKitはRedisでノード間ルーティングしているが、
   自社の`aruaru-db`等で代替できるか、Redisをそのまま採用するかを検討する。
3. **シグナリングプロトコル設計**: LiveKitはgRPC+Protocol Buffersだが、
   `RPoem`(GraphQL Federation、Cosmo互換)側との親和性を踏まえた設計を検討する。
4. **Simulcast(複数解像度レイヤー)対応方針**: 実装の複雑度とのバランスを
   踏まえ、初期実装のスコープに含めるか段階的に対応するかを検討する。
5. **`open-tv-chat`側の翻訳エンジンとの統合方式**: LiveKitのAgentディスパッチ
   相当の仕組み(参加者の音声トラックへ翻訳パイプラインをフックする仕組み)を
   どう自前設計するかを検討する。

## 次回再開ポイント

ユーザー指示により「じっくりゆっくり」調査を進める方針のため、次回セッションでは
上記1(WebRTCスタックの選定)から着手する想定。GitHub上のwebrtc-rs/str0mそれぞれの
Issue・スター数・最終更新・実運用事例(本番採用しているプロジェクトの有無)を
調査した上で、比較表を`README.md`に追記してから次(2)以降へ進める。
