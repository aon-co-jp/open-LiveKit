# webrtc-dtls 0.12.0 への差分 / Patches

上流: https://github.com/webrtc-rs/webrtc (`dtls`, MIT OR Apache-2.0)。ライセンス表記は `LICENSE-*` のまま。

**目的**: `p256` / `p384`(→ `elliptic-curve` 0.13 → `crypto-bigint` 0.5.5)を依存から外す
(RUSTSEC-2026-0387)。DTLSの手順そのものは変えていない。

- `Cargo.toml`: `p256`・`p384` を削除。
- `src/curve/named_curve.rs`: P-256/P-384のECDH鍵の生成を `ring::agreement` へ。`agree_ephemeral` は鍵を
  消費するため、最初の計算結果を相手の公開鍵とともに保持し、同じ相手なら再利用する(`RingEcdh`)。
- `src/prf/mod.rs`: 共有秘密の計算を `RingEcdh::agree` へ。相手の公開鍵の検証(曲線上の点か)は ring が行う。
- `src/error.rs`: `P256Error` を文字列の入れ物に変更し `ring::error::Unspecified` から変換。
- `src/crypto/crypto_cbc.rs`: `subtle::ConstantTimeEq` を直接使う。

署名・検証(ECDSA)は元から `ring` で、変更していない。X25519(`x25519-dalek`)も変更なし。
上流が `crypto-bigint` 0.7 系へ追従したら、この `[patch.crates-io]` ごと外す。
