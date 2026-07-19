use common::NodeId;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

pub type PublicKey = VerifyingKey;

pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        self.signing_key.verifying_key()
    }

    pub fn node_id(&self) -> NodeId {
        NodeId(hex::encode(self.public_key().to_bytes()))
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.signing_key.sign(data).to_bytes().to_vec()
    }

    pub fn verify(&self, data: &[u8], sig: &[u8]) -> bool {
        let signature = match Signature::from_slice(sig) {
            Ok(s) => s,
            Err(_) => return false,
        };
        self.signing_key
            .verifying_key()
            .verify_strict(data, &signature)
            .is_ok()
    }
}

#[async_trait::async_trait]
pub trait CryptoProvider: Send + Sync {
    async fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>>;
    async fn verify(&self, data: &[u8], signature: &[u8]) -> anyhow::Result<bool>;
}

pub struct Ed25519CryptoProvider {
    keypair: KeyPair,
}

impl Ed25519CryptoProvider {
    pub fn new(keypair: KeyPair) -> Self {
        Self { keypair }
    }
}

#[async_trait::async_trait]
impl CryptoProvider for Ed25519CryptoProvider {
    async fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let signature = self.keypair.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> anyhow::Result<bool> {
        let signature = Signature::from_slice(signature)
            .map_err(|e| anyhow::anyhow!("invalid signature bytes: {}", e))?;
        Ok(self
            .keypair
            .public_key()
            .verify_strict(data, &signature)
            .is_ok())
    }
}

pub struct DefaultCryptoProvider {
    pub keypair: KeyPair,
}

impl DefaultCryptoProvider {
    pub fn new() -> Self {
        Self {
            keypair: KeyPair::generate(),
        }
    }
}

impl Default for DefaultCryptoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CryptoProvider for DefaultCryptoProvider {
    async fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let signature = self.keypair.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> anyhow::Result<bool> {
        let signature = Signature::from_slice(signature)
            .map_err(|e| anyhow::anyhow!("invalid signature bytes: {}", e))?;
        Ok(self
            .keypair
            .public_key()
            .verify_strict(data, &signature)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation_produces_non_empty_keys() {
        let kp = KeyPair::generate();
        let bytes = kp.to_bytes();
        assert_eq!(bytes.len(), 32);
        assert!(bytes.iter().any(|&b| b != 0));
        assert!(!kp.node_id().0.is_empty());
    }

    #[test]
    fn test_sign_produces_64_byte_signature() {
        let kp = KeyPair::generate();
        let sig = kp.signing_key.sign(b"test data");
        assert_eq!(sig.to_bytes().len(), 64);
    }

    #[test]
    fn test_signing_is_deterministic() {
        let kp = KeyPair::generate();
        let sig1 = kp.signing_key.sign(b"test data");
        let sig2 = kp.signing_key.sign(b"test data");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_verification_succeeds_for_correct_data() {
        let kp = KeyPair::generate();
        let sig = kp.signing_key.sign(b"test data");
        assert!(kp.public_key().verify_strict(b"test data", &sig).is_ok());
    }

    #[test]
    fn test_verification_fails_for_wrong_data() {
        let kp = KeyPair::generate();
        let sig = kp.signing_key.sign(b"test data");
        assert!(kp.public_key().verify_strict(b"wrong data", &sig).is_err());
    }

    #[test]
    fn test_verification_fails_for_wrong_signature() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let sig = kp1.signing_key.sign(b"test data");
        assert!(kp2.public_key().verify_strict(b"test data", &sig).is_err());
    }

    #[test]
    fn test_keypair_serialization_roundtrip() {
        let kp = KeyPair::generate();
        let bytes = kp.to_bytes();
        let kp2 = KeyPair::from_seed(&bytes);
        assert_eq!(kp.to_bytes(), kp2.to_bytes());
        assert_eq!(kp.public_key(), kp2.public_key());
    }

    #[test]
    fn test_node_id_derivation_from_public_key() {
        let kp = KeyPair::generate();
        let node_id = kp.node_id();
        let expected = hex::encode(kp.public_key().to_bytes());
        assert_eq!(node_id.0, expected);
        assert_eq!(node_id.0.len(), 64);
    }

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        assert!(!kp.node_id().0.is_empty());
    }

    #[tokio::test]
    async fn test_ed25519_crypto_provider_sign_verify() {
        let provider = Ed25519CryptoProvider::new(KeyPair::generate());
        let sig = provider.sign(b"hello").await.unwrap();
        assert_eq!(sig.len(), 64);
        let valid = provider.verify(b"hello", &sig).await.unwrap();
        assert!(valid);
    }

    #[tokio::test]
    async fn test_ed25519_crypto_provider_verify_fails_wrong_data() {
        let provider = Ed25519CryptoProvider::new(KeyPair::generate());
        let sig = provider.sign(b"hello").await.unwrap();
        let valid = provider.verify(b"wrong", &sig).await.unwrap();
        assert!(!valid);
    }

    #[tokio::test]
    async fn test_default_crypto_provider_sign_verify() {
        let provider = DefaultCryptoProvider::new();
        let sig = provider.sign(b"hello").await.unwrap();
        assert_eq!(sig.len(), 64);
        let valid = provider.verify(b"hello", &sig).await.unwrap();
        assert!(valid);
    }

    #[tokio::test]
    async fn test_default_crypto_provider_verify_fails_wrong_signature() {
        let provider = DefaultCryptoProvider::new();
        let sig = provider.sign(b"hello").await.unwrap();
        let provider2 = DefaultCryptoProvider::new();
        let valid = provider2.verify(b"hello", &sig).await.unwrap();
        assert!(!valid);
    }
}
