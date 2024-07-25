use anyhow::Result;
use hmac::{Hmac, Mac};
use jwt::{RegisteredClaims, SignWithKey, VerifyWithKey};
use sha2::Sha256;

pub fn proceess_jwt_sign(sub: String, aud: String, exp: u64) -> Result<String> {
    let claims = RegisteredClaims {
        subject: Some(sub),
        audience: Some(aud),
        expiration: Some(exp),
        ..Default::default()
    };
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(b"A!76mvQ4vFzB").map_err(|_e| anyhow::anyhow!("Invalid key"))?;

    let signed_token = claims
        .sign_with_key(&key)
        .map_err(|_e| anyhow::anyhow!("Sign failed"))?;

    Ok(signed_token)
}

pub fn process_jwt_verify(token: String) -> Result<()> {
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(b"A!76mvQ4vFzB").map_err(|_e| anyhow::anyhow!("Invalid key"))?;
    let claims: RegisteredClaims = VerifyWithKey::verify_with_key(token.as_str(), &key)
        .map_err(|_e| anyhow::anyhow!("Parse failed"))?;

    claims.subject.ok_or(anyhow::anyhow!("Missing subject"))?;
    Ok(())
}
