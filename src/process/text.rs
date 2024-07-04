use std::{fs, io::Read, path::Path};

use crate::{cli::TextCryptoFormat, get_reader, process_genpass, TextSignFormat};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305,
};

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

pub trait TextSign {
    // 动态分派
    /// sign the data from reader and return the signature
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    // 静态分派
    /// fn verify<R: Read>(&self, reader: R, signature: &[u8]) -> Result<bool>;
    fn verify(&self, reader: impl Read, signature: &[u8]) -> Result<bool>;
}

pub trait TextEncrypt {
    fn encrypt(&self, reader: &mut impl Read) -> Result<Vec<u8>>;
}

pub trait TextDecrypt {
    fn decrypt(&self, reader: &mut impl Read) -> Result<Vec<u8>>;
}

pub struct Chacha20 {
    key: [u8; 32],
    nonce: [u8; 12],
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    /// verify the data from the reader with the signature
    key: VerifyingKey,
}

pub trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    // owned value在使用时如果是可变的，要加mut关键字，在trait声明时不需要加
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);
        let hash = hash.as_bytes();
        Ok(hash == sig)
    }
}

pub fn process_text_generate(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        // from_bytes的参数是pub type SignatureBytes = [u8; Signature::BYTE_SIZE];它本身也是个u8数组
        // 这里调用try_into()，编译器会尝试将&[u8]转化为SignatureBytes，是可以转换的
        let sig = Signature::from_bytes(sig.try_into()?);
        let ret = self.key.verify(&buf, &sig).is_ok();
        Ok(ret)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        // Vec<u8>不支持?，没有实现FromResidual<R>，要转换为&[u8] slice才能使用
        // 同时只获取32个字节（因为生成密码的时候我们就只用了32个字节，blake3.txt我们是通过cargo run -- genpass -l 32 > fixtures/blake3.txt生成的）能去除结尾多余的换行符之类的符号
        let key = &key[..32];
        let key = key.try_into()?;
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl Chacha20 {
    fn load(keypath: impl AsRef<Path>, noncepath: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(keypath)?;
        let nonce = fs::read(noncepath)?;
        Self::try_new(&key, &nonce)
    }

    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        Self { key, nonce }
    }

    pub fn try_new(key: &[u8], nonce: &[u8]) -> Result<Self> {
        let key = &key[..32];
        // let nonce = ChaCha20Poly1305::generate_nonce(&mut chacha20poly1305::aead::OsRng);   // 96-bits; unique per message
        let nonce: &[u8] = &nonce[..12];
        let encrypt = Chacha20::new(key.try_into()?, nonce.try_into()?);
        Ok(encrypt)
    }
}

// impl KeyLoader for Chacha20 {
//     fn load(path: impl AsRef<Path>) -> Result<Self> {
//         let key = fs::read(path)?;
//         Self::try_new(&key)
//     }
// }

impl TextEncrypt for Chacha20 {
    fn encrypt(&self, reader: &mut impl Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&self.key[..32])?;
        // let nonce = ChaCha20Poly1305::generate_nonce(&mut chacha20poly1305::aead::OsRng); // 96-bits; unique per message
        let nonce = &self.nonce[..12];
        let ciphertext = cipher
            .encrypt(nonce.into(), buf.as_ref())
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(ciphertext)
    }
}

impl TextDecrypt for Chacha20 {
    fn decrypt(&self, reader: &mut impl Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&self.key[..32])?;
        let input = URL_SAFE_NO_PAD.decode(&buf)?;
        let nonce = &self.nonce[..12];
        let plaintext = cipher
            .decrypt(nonce.into(), input.as_ref())
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(plaintext)
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key().to_bytes().to_vec();
        let sk = sk.to_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        let signer = Ed25519Signer::new(key);
        Ok(signer)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}
impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        let signer = Ed25519Verifier::new(key);
        Ok(signer)
    }
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };
    let signed = URL_SAFE_NO_PAD.encode(signed);
    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> anyhow::Result<bool> {
    let mut reader = get_reader(input)?;
    let sig = URL_SAFE_NO_PAD.decode(sig)?;
    let verified = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
    };
    Ok(verified)
}

pub fn process_text_encrypt(
    input: &str,
    key: &str,
    nonce: &str,
    format: TextCryptoFormat,
) -> Result<String> {
    let mut reader: Box<dyn Read> = get_reader(input)?;
    let encrypted = match format {
        TextCryptoFormat::Chacha20 => {
            let encrypt = Chacha20::load(key, nonce)?;
            encrypt.encrypt(&mut reader)?
        }
    };
    let encrypted = URL_SAFE_NO_PAD.encode(encrypted);
    Ok(encrypted)
}

pub fn process_text_decrypt(
    input: &str,
    key: &str,
    nonce: &str,
    format: TextCryptoFormat,
) -> Result<String> {
    let mut reader: Box<dyn Read> = get_reader(input)?;
    let decrypted = match format {
        TextCryptoFormat::Chacha20 => {
            let decrypt = Chacha20::load(key, nonce)?;
            decrypt.decrypt(&mut reader)?
        }
    };
    let decrypted = String::from_utf8(decrypted)?;
    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake_sign_verify() -> Result<()> {
        let blake3 = Blake3::load("fixtures/blake3.txt")?;
        let data = b"hello";
        let sig = blake3.sign(&mut &data[..]).unwrap();
        assert!(blake3.verify(&data[..], &sig).unwrap());
        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> Result<()> {
        let sk = Ed25519Signer::load("fixtures/ed25519.sk")?;
        let pk = Ed25519Verifier::load("fixtures/ed25519.pk")?;
        let data = b"hello";
        //&data[..]是个切片，sign传的是切片的可变引用，verifys传的是所有权
        let sig = sk.sign(&mut &data[..])?;
        assert!(pk.verify(&data[..], &sig)?);
        Ok(())
    }
}
