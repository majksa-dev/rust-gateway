use anyhow::{anyhow, Error, Result};
use openidconnect::core::CoreJwsSigningAlgorithm;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The algorithms supported for signing/verifying JWTs
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Algorithm {
    /// HMAC using SHA-256
    #[default]
    HS256,
    /// HMAC using SHA-384
    HS384,
    /// HMAC using SHA-512
    HS512,

    /// ECDSA using SHA-256
    ES256,
    /// ECDSA using SHA-384
    ES384,

    /// RSASSA-PKCS1-v1_5 using SHA-256
    RS256,
    /// RSASSA-PKCS1-v1_5 using SHA-384
    RS384,
    /// RSASSA-PKCS1-v1_5 using SHA-512
    RS512,

    /// RSASSA-PSS using SHA-256
    PS256,
    /// RSASSA-PSS using SHA-384
    PS384,
    /// RSASSA-PSS using SHA-512
    PS512,

    /// Edwards-curve Digital Signature Algorithm (EdDSA)
    EdDSA,
}

impl FromStr for Algorithm {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "HS256" => Ok(Algorithm::HS256),
            "HS384" => Ok(Algorithm::HS384),
            "HS512" => Ok(Algorithm::HS512),
            "ES256" => Ok(Algorithm::ES256),
            "ES384" => Ok(Algorithm::ES384),
            "RS256" => Ok(Algorithm::RS256),
            "RS384" => Ok(Algorithm::RS384),
            "PS256" => Ok(Algorithm::PS256),
            "PS384" => Ok(Algorithm::PS384),
            "PS512" => Ok(Algorithm::PS512),
            "RS512" => Ok(Algorithm::RS512),
            "EdDSA" => Ok(Algorithm::EdDSA),
            _ => Err(anyhow!("Invalid algorithm")),
        }
    }
}

impl From<Algorithm> for CoreJwsSigningAlgorithm {
    fn from(value: Algorithm) -> Self {
        match value {
            Algorithm::HS256 => Self::HmacSha256,
            Algorithm::HS384 => Self::HmacSha384,
            Algorithm::HS512 => Self::HmacSha512,
            Algorithm::ES256 => Self::EcdsaP256Sha256,
            Algorithm::ES384 => Self::EcdsaP384Sha384,
            Algorithm::RS256 => Self::RsaSsaPkcs1V15Sha256,
            Algorithm::RS384 => Self::RsaSsaPkcs1V15Sha384,
            Algorithm::RS512 => Self::RsaSsaPkcs1V15Sha512,
            Algorithm::PS256 => Self::RsaSsaPssSha256,
            Algorithm::PS384 => Self::RsaSsaPssSha384,
            Algorithm::PS512 => Self::RsaSsaPssSha512,
            Algorithm::EdDSA => Self::EdDsaEd25519,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_algorithm_enum_from_str() {
        assert!(Algorithm::from_str("HS256").is_ok());
        assert!(Algorithm::from_str("HS384").is_ok());
        assert!(Algorithm::from_str("HS512").is_ok());
        assert!(Algorithm::from_str("RS256").is_ok());
        assert!(Algorithm::from_str("RS384").is_ok());
        assert!(Algorithm::from_str("RS512").is_ok());
        assert!(Algorithm::from_str("PS256").is_ok());
        assert!(Algorithm::from_str("PS384").is_ok());
        assert!(Algorithm::from_str("PS512").is_ok());
        assert!(Algorithm::from_str("").is_err());
    }
}
