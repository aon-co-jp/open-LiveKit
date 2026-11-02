use rand_core::OsRng; // requires 'getrandom' feature

use crate::error::*;

// https://www.iana.org/assignments/tls-parameters/tls-parameters.xml#tls-parameters-8
#[repr(u16)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NamedCurve {
    Unsupported = 0x0000,
    P256 = 0x0017,
    P384 = 0x0018,
    X25519 = 0x001d,
}

impl From<u16> for NamedCurve {
    fn from(val: u16) -> Self {
        match val {
            0x0017 => NamedCurve::P256,
            0x0018 => NamedCurve::P384,
            0x001d => NamedCurve::X25519,
            _ => NamedCurve::Unsupported,
        }
    }
}

pub(crate) enum NamedCurvePrivateKey {
    EphemeralSecretP256(RingEcdh),
    EphemeralSecretP384(RingEcdh),
    StaticSecretX25519(x25519_dalek::StaticSecret),
}

/// ringのECDH秘密鍵。`agree_ephemeral`は鍵を消費するので、最初の計算結果を相手の公開鍵と
/// 一緒に保持し、同じ相手での再計算には保持した値を返す(p256/crypto-bigintを使わないための差し替え)。
pub(crate) struct RingEcdh {
    key: std::sync::Mutex<Option<ring::agreement::EphemeralPrivateKey>>,
    done: std::sync::Mutex<Option<(Vec<u8>, Vec<u8>)>>,
}

impl RingEcdh {
    fn generate(alg: &'static ring::agreement::Algorithm) -> Result<(Self, Vec<u8>)> {
        let rng = ring::rand::SystemRandom::new();
        let key = ring::agreement::EphemeralPrivateKey::generate(alg, &rng)?;
        let public = key.compute_public_key()?.as_ref().to_vec();
        Ok((Self { key: std::sync::Mutex::new(Some(key)), done: std::sync::Mutex::new(None) }, public))
    }

    pub(crate) fn agree(&self, alg: &'static ring::agreement::Algorithm, peer: &[u8]) -> Result<Vec<u8>> {
        let mut done = self.done.lock().unwrap();
        if let Some((p, secret)) = done.as_ref() {
            return if p.as_slice() == peer { Ok(secret.clone()) } else { Err(Error::Other("ECDH key already used with another peer key".into())) };
        }
        let key = self.key.lock().unwrap().take().ok_or_else(|| Error::Other("ECDH key already consumed".into()))?;
        let secret = ring::agreement::agree_ephemeral(
            key,
            &ring::agreement::UnparsedPublicKey::new(alg, peer),
            |s| s.to_vec(),
        )?;
        *done = Some((peer.to_vec(), secret.clone()));
        Ok(secret)
    }
}

pub struct NamedCurveKeypair {
    pub(crate) curve: NamedCurve,
    pub(crate) public_key: Vec<u8>,
    pub(crate) private_key: NamedCurvePrivateKey,
}

fn elliptic_curve_keypair(curve: NamedCurve) -> Result<NamedCurveKeypair> {
    let (public_key, private_key) = match curve {
        NamedCurve::P256 => {
            let (secret_key, public_key) = RingEcdh::generate(&ring::agreement::ECDH_P256)?;
            (public_key, NamedCurvePrivateKey::EphemeralSecretP256(secret_key))
        }
        NamedCurve::P384 => {
            let (secret_key, public_key) = RingEcdh::generate(&ring::agreement::ECDH_P384)?;
            (public_key, NamedCurvePrivateKey::EphemeralSecretP384(secret_key))
        }
        NamedCurve::X25519 => {
            let secret_key = x25519_dalek::StaticSecret::random_from_rng(OsRng);
            let public_key = x25519_dalek::PublicKey::from(&secret_key);
            (
                public_key.as_bytes().to_vec(),
                NamedCurvePrivateKey::StaticSecretX25519(secret_key),
            )
        }
        _ => return Err(Error::ErrInvalidNamedCurve),
    };

    Ok(NamedCurveKeypair {
        curve,
        public_key,
        private_key,
    })
}

impl NamedCurve {
    pub fn generate_keypair(&self) -> Result<NamedCurveKeypair> {
        match *self {
            NamedCurve::X25519 => elliptic_curve_keypair(NamedCurve::X25519),
            NamedCurve::P256 => elliptic_curve_keypair(NamedCurve::P256),
            NamedCurve::P384 => elliptic_curve_keypair(NamedCurve::P384),
            _ => Err(Error::ErrInvalidNamedCurve),
        }
    }
}
