use algorithm::Algorithm;
use anyhow::{anyhow, bail, Context, Result};
use essentials::warn;
use header::Header;
use openidconnect::core::{CoreJsonWebKey, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::IssuerUrl;
use openidconnect::{JsonWebKey, JsonWebKeyId};
use serde::de::DeserializeOwned;
use serialization::DecodedJwtPartClaims;
use std::collections::HashMap;
use validation::{validate, Validation};

mod algorithm;
mod header;
mod serialization;
mod validation;

macro_rules! expect_two {
    ($iter:expr) => {{
        let mut i = $iter;
        match (i.next(), i.next(), i.next()) {
            (Some(first), Some(second), None) => (first, second),
            _ => bail!("Invalid token"),
        }
    }};
}

fn verify_signature<'a>(
    token: &'a str,
    validation: &Validation,
    keys: &[CoreJsonWebKey],
) -> Result<&'a str> {
    if validation.algorithms.is_empty() {
        bail!("Missing algorithm");
    }
    let (signature, message) = expect_two!(token.rsplitn(2, '.'));
    let (payload, header) = expect_two!(message.rsplitn(2, '.'));
    let header = Header::from_encoded(header)?;
    let key_id = JsonWebKeyId::new(header.kid.ok_or_else(|| anyhow!("Missing key ID"))?);
    let key = keys
        .iter()
        .find(|key| key.key_id().is_some_and(|kid| *kid == key_id))
        .ok_or_else(|| anyhow!("Key not allowed"))?;
    key.verify_signature(&header.alg.into(), message.as_bytes(), signature.as_bytes())
        .with_context(|| format!("Failed to verify signature with key ID: {:?}", key_id))?;

    Ok(payload)
}

fn decode<T: DeserializeOwned>(
    token: &str,
    keys: &[CoreJsonWebKey],
    validation: &Validation,
) -> Result<T> {
    let claims = verify_signature(token, validation, keys)?;
    let decoded_claims = DecodedJwtPartClaims::from_jwt_part_claims(claims)?;
    let claims = decoded_claims.deserialize()?;
    validate(decoded_claims.deserialize()?, validation)?;
    Ok(claims)
}

pub async fn fetch_keys(issuer_url: IssuerUrl) -> Result<Vec<CoreJsonWebKey>> {
    let metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client).await?;
    Ok(metadata.jwks().keys().clone())
}

pub async fn parse_token(token: &str, keys: &[CoreJsonWebKey]) -> Option<HashMap<String, String>> {
    decode::<HashMap<String, String>>(token, keys, &Validation::new(Algorithm::HS256))
        .map_err(|e| warn!(error = %e, "Failed to parse token"))
        .ok()
}
