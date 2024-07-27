use anyhow::{anyhow, Result};
use essentials::warn;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{DecodingKey, Validation};
use serde::de::DeserializeOwned;
use serde_json::Value;

fn decode<T: DeserializeOwned>(token: &str, keys: &[Jwk], validation: &Validation) -> Result<T> {
    let header = jsonwebtoken::decode_header(token)?;
    let key_id = header.kid.ok_or_else(|| anyhow!("Missing key ID"))?;
    let key = keys
        .iter()
        .find(|key| key.common.key_id.as_ref().is_some_and(|kid| *kid == key_id))
        .ok_or_else(|| anyhow!("Key not allowed"))?;
    let decoding_key = DecodingKey::from_jwk(key)?;
    let data = jsonwebtoken::decode(token, &decoding_key, validation)?;
    Ok(data.claims)
}

pub async fn parse_token(token: &str, keys: &[Jwk], validation: &Validation) -> Option<Value> {
    decode::<Value>(token, keys, validation)
        .map_err(|e| warn!(error = %e, "Failed to parse token"))
        .ok()
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{jwk::JwkSet, Algorithm, EncodingKey, Header};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::auth::ClaimParser;

    use super::*;

    #[tokio::test]
    async fn test_unsigned_keys() {
        let jwk = serde_json::from_value(json!({
        "e": "AQAB",
        "kid": "3d580f0af7ace698a0cee7f230bca594dc6dbb55",
        "kty": "RSA",
        "n": "rhgQZT3t9MgNBv9_4qE58CLCbDfEaRd9HgPd_Zmjg1TIYjHh1UgMPVeVekyU2JiuUZPbnlEbv8WUsxyNNQJfATvfMbXaUcrePSdW32zIaMOeTbn0VXZ3tqx5IyiP0IfJt-kT9MilGAkeJn8me7x5_uNGOpiPCWQaxFxTikVUtGO5AbGh2PTULzKjVjZWwQrPB1fqEe6Ar6Im-3RcZ-zOd3N2ThgQEzLLRe4RE6bSvBQUuxX9o_AkY0SCVZZB2VhjQYBN3EUFmKsD46rrneBn64Vduy3jWtBYXA1avDRCl0Y8yQEBOrtgikEz_hog4O4EKP5mAVSf8Iyfl_RMdxrOAQ",
        "alg": "RS256",
        "use": "sig"
        })).unwrap();
        let keys = [jwk];
        assert!(
            parse_token("", keys.as_slice(), &Validation::new(Algorithm::RS256))
                .await
                .is_none()
        );
        assert!(parse_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c", keys.as_slice(), &Validation::new(Algorithm::RS256)).await.is_none());
        assert!(parse_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsIng1dCI6Ikh5cTROQVRBanNucUM3bWRydEFoaHJDUjJfUSIsImtpZCI6IjFGMkFCODM0MDRDMDhFQzlFQTBCQjk5REFFRDAyMTg2QjA5MURCRjQifQ.eyJqdGkiOiJkNjhlNzc3MS1hMTc3LTRhNTktOTFmYi0zMjkzOTJlMzIwNmQiLCJzdWIiOiJyZXBvOmNzYXMtb3BzL2NpY2Qtc2FtcGxlcy1hcHBzOnJlZjpyZWZzL2hlYWRzL3NiLXRlc3REb3BvVXJpIiwiYXVkIjoiaHR0cHM6Ly9naXRodWIuY29tL2NzYXMtb3BzIiwicmVmIjoicmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsInNoYSI6IjVjYTYxMmIzNjFhNWJlNmJkZDQxZjY5MDEwYjMwMDljOWFmMjIzM2IiLCJyZXBvc2l0b3J5IjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMiLCJyZXBvc2l0b3J5X293bmVyIjoiY3Nhcy1vcHMiLCJyZXBvc2l0b3J5X293bmVyX2lkIjoiNzU5ODA4NTYiLCJydW5faWQiOiI3MTk2MDQwMDE3IiwicnVuX251bWJlciI6IjQzIiwicnVuX2F0dGVtcHQiOiIxIiwicmVwb3NpdG9yeV92aXNpYmlsaXR5IjoiaW50ZXJuYWwiLCJyZXBvc2l0b3J5X2lkIjoiNDg5MzkwNTcwIiwiYWN0b3JfaWQiOiIxMjk3NDc5NDYiLCJhY3RvciI6InN0YW5pc2xhdmJlYmVqLWV4dDQzMzQ1Iiwid29ya2Zsb3ciOiJET1BPIiwiaGVhZF9yZWYiOiIiLCJiYXNlX3JlZiI6IiIsImV2ZW50X25hbWUiOiJ3b3JrZmxvd19kaXNwYXRjaCIsInJlZl9wcm90ZWN0ZWQiOiJmYWxzZSIsInJlZl90eXBlIjoiYnJhbmNoIiwid29ya2Zsb3dfcmVmIjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMvLmdpdGh1Yi93b3JrZmxvd3MvZG9wby55bWxAcmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsIndvcmtmbG93X3NoYSI6IjVjYTYxMmIzNjFhNWJlNmJkZDQxZjY5MDEwYjMwMDljOWFmMjIzM2IiLCJqb2Jfd29ya2Zsb3dfcmVmIjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMvLmdpdGh1Yi93b3JrZmxvd3MvZG9wby55bWxAcmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsImpvYl93b3JrZmxvd19zaGEiOiI1Y2E2MTJiMzYxYTViZTZiZGQ0MWY2OTAxMGIzMDA5YzlhZjIyMzNiIiwicnVubmVyX2Vudmlyb25tZW50Ijoic2VsZi1ob3N0ZWQiLCJlbnRlcnByaXNlIjoiY2Vza2Etc3Bvcml0ZWxuYSIsImlzcyI6Imh0dHBzOi8vdG9rZW4uYWN0aW9ucy5naXRodWJ1c2VyY29udGVudC5jb20iLCJuYmYiOjE3MDI0NzQxNTAsImV4cCI6MTcwMjQ3NTA1MCwiaWF0IjoxNzAyNDc0NzUwfQ.iD7_wPpoWdfRI1J6MDHVB0OJQWiVMqg0l5o4m94n1Mre_zMsgXcoXPdU7nFyDppZ7vFNZsa1SJh-Lu4Q0EGBznjNxiv724kbuS5ZfXsrH2wx2SPKjt5BZnDiUrgCjdiHBk-0PkIdxw2LfZ9VlUqu_Abd-VbdRWzKbt2AYFZKH9lh24OadgIQGlxMfz8tkAS1HTsXuway_qqhAWtyneXEDszWWTvwehkUw77-o5pYrMX72Voxwyql1HjztSw1mrDGYeZ5IiU1au_xUC97saZHkRZh4_ZReJYekODih0i7Xwla8w8tDxSJKuZ2a0y-uJYVGRmNQFcZQ1bcYQth6otcBA", keys.as_slice(), &Validation::new(Algorithm::RS256)).await.is_none());
    }

    #[tokio::test]
    async fn test_signed_key() {
        let jwk = serde_json::from_value(json!({
            "kty": "RSA",
            "alg": "RS256",
            "use": "sig",
            "kid": "1F2AB83404C08EC9EA0BB99DAED02186B091DBF4",
            "n": "u8zSYn5JR_O5yywSeOhmWWd7OMoLblh4iGTeIhTOVon-5e54RK30YQDeUCjpb9u3vdHTO7XS7i6EzkwLbsUOir27uhqoFGGWXSAZrPocOobSFoLC5l0NvSKRqVtpoADOHcAh59vLbr8dz3xtEEGx_qlLTzfFfWiCIYWiy15C2oo1eNPxzQfOvdu7Yet6Of4musV0Es5_mNETpeHOVEri8PWfxzw485UHIj3socl4Lk_I3iDyHfgpT49tIJYhHE5NImLNdwMha1cBCIbJMy1dJCfdoK827Hi9qKyBmftNQPhezGVRsOjsf2BfUGzGP5pCGrFBjEOcLhj_3j-TJebgvQ",
            "e": "AQAB",
            "x5c": [
            "MIIDrDCCApSgAwIBAgIQAP4blP36Q3WmMOhWf0RBMzANBgkqhkiG9w0BAQsFADA2MTQwMgYDVQQDEyt2c3RzLXZzdHNnaHJ0LWdoLXZzby1vYXV0aC52aXN1YWxzdHVkaW8uY29tMB4XDTIzMTAyNDE0NTI1NVoXDTI1MTAyNDE1MDI1NVowNjE0MDIGA1UEAxMrdnN0cy12c3RzZ2hydC1naC12c28tb2F1dGgudmlzdWFsc3R1ZGlvLmNvbTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBALvM0mJ+SUfzucssEnjoZllnezjKC25YeIhk3iIUzlaJ/uXueESt9GEA3lAo6W/bt73R0zu10u4uhM5MC27FDoq9u7oaqBRhll0gGaz6HDqG0haCwuZdDb0ikalbaaAAzh3AIefby26/Hc98bRBBsf6pS083xX1ogiGFosteQtqKNXjT8c0Hzr3bu2Hrejn+JrrFdBLOf5jRE6XhzlRK4vD1n8c8OPOVByI97KHJeC5PyN4g8h34KU+PbSCWIRxOTSJizXcDIWtXAQiGyTMtXSQn3aCvNux4vaisgZn7TUD4XsxlUbDo7H9gX1Bsxj+aQhqxQYxDnC4Y/94/kyXm4L0CAwEAAaOBtTCBsjAOBgNVHQ8BAf8EBAMCBaAwCQYDVR0TBAIwADAdBgNVHSUEFjAUBggrBgEFBQcDAQYIKwYBBQUHAwIwNgYDVR0RBC8wLYIrdnN0cy12c3RzZ2hydC1naC12c28tb2F1dGgudmlzdWFsc3R1ZGlvLmNvbTAfBgNVHSMEGDAWgBSmWMP5CXuaSzoLKwcLXYZnoeCJmDAdBgNVHQ4EFgQUpljD+Ql7mks6CysHC12GZ6HgiZgwDQYJKoZIhvcNAQELBQADggEBAINwybFwYpXJkvauL5QbtrykIDYeP8oFdVIeVY8YI9MGfx7OwWDsNBVXv2B62zAZ49hK5G87++NmFI/FHnGOCISDYoJkRSCy2Nbeyr7Nx2VykWzUQqHLZfvr5KqW4Gj1OFHUqTl8lP3FWDd/P+lil3JobaSiICQshgF0GnX2a8ji8mfXpJSP20gzrLw84brmtmheAvJ9X/sLbM/RBkkT6g4NV2QbTMqo6k601qBNQBsH+lTDDWPCkRoAlW6a0z9bWIhGHWJ2lcR70zagcxIVl5/Fq35770/aMGroSrIx3JayOEqsvgIthYBKHzpT2VFwUz1VpBpNVJg9/u6jCwLY7QA="
            ],
            "x5t": "Hyq4NATAjsnqC7mdrtAhhrCR2_Q"
        })).unwrap();
        let keys = [jwk];
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsIng1dCI6Ikh5cTROQVRBanNucUM3bWRydEFoaHJDUjJfUSIsImtpZCI6IjFGMkFCODM0MDRDMDhFQzlFQTBCQjk5REFFRDAyMTg2QjA5MURCRjQifQ.eyJqdGkiOiJkNjhlNzc3MS1hMTc3LTRhNTktOTFmYi0zMjkzOTJlMzIwNmQiLCJzdWIiOiJyZXBvOmNzYXMtb3BzL2NpY2Qtc2FtcGxlcy1hcHBzOnJlZjpyZWZzL2hlYWRzL3NiLXRlc3REb3BvVXJpIiwiYXVkIjoiaHR0cHM6Ly9naXRodWIuY29tL2NzYXMtb3BzIiwicmVmIjoicmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsInNoYSI6IjVjYTYxMmIzNjFhNWJlNmJkZDQxZjY5MDEwYjMwMDljOWFmMjIzM2IiLCJyZXBvc2l0b3J5IjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMiLCJyZXBvc2l0b3J5X293bmVyIjoiY3Nhcy1vcHMiLCJyZXBvc2l0b3J5X293bmVyX2lkIjoiNzU5ODA4NTYiLCJydW5faWQiOiI3MTk2MDQwMDE3IiwicnVuX251bWJlciI6IjQzIiwicnVuX2F0dGVtcHQiOiIxIiwicmVwb3NpdG9yeV92aXNpYmlsaXR5IjoiaW50ZXJuYWwiLCJyZXBvc2l0b3J5X2lkIjoiNDg5MzkwNTcwIiwiYWN0b3JfaWQiOiIxMjk3NDc5NDYiLCJhY3RvciI6InN0YW5pc2xhdmJlYmVqLWV4dDQzMzQ1Iiwid29ya2Zsb3ciOiJET1BPIiwiaGVhZF9yZWYiOiIiLCJiYXNlX3JlZiI6IiIsImV2ZW50X25hbWUiOiJ3b3JrZmxvd19kaXNwYXRjaCIsInJlZl9wcm90ZWN0ZWQiOiJmYWxzZSIsInJlZl90eXBlIjoiYnJhbmNoIiwid29ya2Zsb3dfcmVmIjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMvLmdpdGh1Yi93b3JrZmxvd3MvZG9wby55bWxAcmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsIndvcmtmbG93X3NoYSI6IjVjYTYxMmIzNjFhNWJlNmJkZDQxZjY5MDEwYjMwMDljOWFmMjIzM2IiLCJqb2Jfd29ya2Zsb3dfcmVmIjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMvLmdpdGh1Yi93b3JrZmxvd3MvZG9wby55bWxAcmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsImpvYl93b3JrZmxvd19zaGEiOiI1Y2E2MTJiMzYxYTViZTZiZGQ0MWY2OTAxMGIzMDA5YzlhZjIyMzNiIiwicnVubmVyX2Vudmlyb25tZW50Ijoic2VsZi1ob3N0ZWQiLCJlbnRlcnByaXNlIjoiY2Vza2Etc3Bvcml0ZWxuYSIsImlzcyI6Imh0dHBzOi8vdG9rZW4uYWN0aW9ucy5naXRodWJ1c2VyY29udGVudC5jb20iLCJuYmYiOjE3MDI0NzQxNTAsImV4cCI6MTcwMjQ3NTA1MCwiaWF0IjoxNzAyNDc0NzUwfQ.iD7_wPpoWdfRI1J6MDHVB0OJQWiVMqg0l5o4m94n1Mre_zMsgXcoXPdU7nFyDppZ7vFNZsa1SJh-Lu4Q0EGBznjNxiv724kbuS5ZfXsrH2wx2SPKjt5BZnDiUrgCjdiHBk-0PkIdxw2LfZ9VlUqu_Abd-VbdRWzKbt2AYFZKH9lh24OadgIQGlxMfz8tkAS1HTsXuway_qqhAWtyneXEDszWWTvwehkUw77-o5pYrMX72Voxwyql1HjztSw1mrDGYeZ5IiU1au_xUC97saZHkRZh4_ZReJYekODih0i7Xwla8w8tDxSJKuZ2a0y-uJYVGRmNQFcZQ1bcYQth6otcBA";

        assert!(
            parse_token(token, keys.as_slice(), &Validation::new(Algorithm::RS256))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_verified_key() {
        let rsa_pem = r#"-----BEGIN PRIVATE KEY-----
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
BdfMxd9Hvq9+SlX1OOF4FW/P9FYENA==
-----END PRIVATE KEY-----
"#;
        let encoding_key = EncodingKey::from_rsa_pem(rsa_pem.as_bytes()).unwrap();
        let header = Header {
            kid: Some("abc123".to_string()),
            typ: Some("JWT".to_string()),
            alg: Algorithm::RS256,
            ..Default::default()
        };
        let in_1_day = Utc::now() + Duration::days(1);
        let token = jsonwebtoken::encode(
            &header,
            &json!({
                "sub": "1234567890",
                "name": "John Doe",
                "admin": true,
                "iat": in_1_day.timestamp(),
                "exp": in_1_day.timestamp(),
                "nbf": in_1_day.timestamp(),
                "extra": {
                    "email": "john@doe.com"
                }
            }),
            &encoding_key,
        )
        .unwrap();
        let jwks_json = json!({
            "keys": [
                {
                    "kty": "RSA",
                    "n": "uqaRMwxdRCcGCIAHa5qKUI9sNgQxGitBwZUbkyWIpFaJsJatlkNLich06ybH22ygKkizmNjQlO2tbpRDNVmKBhCOMOQJmv2UfAqDjx9Ap-BMEjSnOGGggzpC_084h08zCOvPwTCw1eK5oOzw5dXfy2X1HiCuJUkAWEeN4VtAH_RVpnBOJPzbtuJ7Sp4JDpRCAkLwBLour51bOTnf7G-ifdmxbfc-hPTojVYToCZIcOhMz-iAVRKvhih1jViLiOQDPq6R11XFI8nQRxSxRYyzZ2TBNxK-DBGPL1d3W2sSWGVw59RMTRU4JgIKsxirhSORbdlJKyWo8quISu_dHqrwbzpFthFl6RPUfToQRgD7TWWVKvSbVPdZdpvaehlo-KOBOg-51mO0qDr7vGQ_CwoXuB4UMjpzGPHSHNLTxjM4xJKUHLnknj_hiYWBO24Z4qdO0VtJEVEsFWHSt8Ubgqb5GdAoI8-6Hh7jvofYzqf9RtXozP_VrbOSH5wgRSeQGckpOjopoVsn2rizBxtpe2xwWYJNG2kAdMIjbvP0MvItpLiC81-5LQ_n1DzfyQW_e04jTySgDblSaz-SDUUZJvTe3IvYNaC5-eH0UvZxMmFefQKBxcQplZEu9DwIp661fgK2pbpsLXqwyLiWhO3BK-5RqT3BvZRCMbDuljQZyDTh8ok",
                    "e": "AQAB",
                    "alg": "RS256",
                    "kid": "abc123",
                    "use": "sig"
                }
            ]
        });
        let keys = serde_json::from_value::<JwkSet>(jwks_json).unwrap().keys;
        assert!(
            parse_token("", keys.as_slice(), &Validation::new(Algorithm::RS256))
                .await
                .is_none()
        );
        assert!(parse_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c", keys.as_slice(), &Validation::new(Algorithm::RS256)).await.is_none());
        assert!(parse_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsIng1dCI6Ikh5cTROQVRBanNucUM3bWRydEFoaHJDUjJfUSIsImtpZCI6IjFGMkFCODM0MDRDMDhFQzlFQTBCQjk5REFFRDAyMTg2QjA5MURCRjQifQ.eyJqdGkiOiJkNjhlNzc3MS1hMTc3LTRhNTktOTFmYi0zMjkzOTJlMzIwNmQiLCJzdWIiOiJyZXBvOmNzYXMtb3BzL2NpY2Qtc2FtcGxlcy1hcHBzOnJlZjpyZWZzL2hlYWRzL3NiLXRlc3REb3BvVXJpIiwiYXVkIjoiaHR0cHM6Ly9naXRodWIuY29tL2NzYXMtb3BzIiwicmVmIjoicmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsInNoYSI6IjVjYTYxMmIzNjFhNWJlNmJkZDQxZjY5MDEwYjMwMDljOWFmMjIzM2IiLCJyZXBvc2l0b3J5IjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMiLCJyZXBvc2l0b3J5X293bmVyIjoiY3Nhcy1vcHMiLCJyZXBvc2l0b3J5X293bmVyX2lkIjoiNzU5ODA4NTYiLCJydW5faWQiOiI3MTk2MDQwMDE3IiwicnVuX251bWJlciI6IjQzIiwicnVuX2F0dGVtcHQiOiIxIiwicmVwb3NpdG9yeV92aXNpYmlsaXR5IjoiaW50ZXJuYWwiLCJyZXBvc2l0b3J5X2lkIjoiNDg5MzkwNTcwIiwiYWN0b3JfaWQiOiIxMjk3NDc5NDYiLCJhY3RvciI6InN0YW5pc2xhdmJlYmVqLWV4dDQzMzQ1Iiwid29ya2Zsb3ciOiJET1BPIiwiaGVhZF9yZWYiOiIiLCJiYXNlX3JlZiI6IiIsImV2ZW50X25hbWUiOiJ3b3JrZmxvd19kaXNwYXRjaCIsInJlZl9wcm90ZWN0ZWQiOiJmYWxzZSIsInJlZl90eXBlIjoiYnJhbmNoIiwid29ya2Zsb3dfcmVmIjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMvLmdpdGh1Yi93b3JrZmxvd3MvZG9wby55bWxAcmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsIndvcmtmbG93X3NoYSI6IjVjYTYxMmIzNjFhNWJlNmJkZDQxZjY5MDEwYjMwMDljOWFmMjIzM2IiLCJqb2Jfd29ya2Zsb3dfcmVmIjoiY3Nhcy1vcHMvY2ljZC1zYW1wbGVzLWFwcHMvLmdpdGh1Yi93b3JrZmxvd3MvZG9wby55bWxAcmVmcy9oZWFkcy9zYi10ZXN0RG9wb1VyaSIsImpvYl93b3JrZmxvd19zaGEiOiI1Y2E2MTJiMzYxYTViZTZiZGQ0MWY2OTAxMGIzMDA5YzlhZjIyMzNiIiwicnVubmVyX2Vudmlyb25tZW50Ijoic2VsZi1ob3N0ZWQiLCJlbnRlcnByaXNlIjoiY2Vza2Etc3Bvcml0ZWxuYSIsImlzcyI6Imh0dHBzOi8vdG9rZW4uYWN0aW9ucy5naXRodWJ1c2VyY29udGVudC5jb20iLCJuYmYiOjE3MDI0NzQxNTAsImV4cCI6MTcwMjQ3NTA1MCwiaWF0IjoxNzAyNDc0NzUwfQ.iD7_wPpoWdfRI1J6MDHVB0OJQWiVMqg0l5o4m94n1Mre_zMsgXcoXPdU7nFyDppZ7vFNZsa1SJh-Lu4Q0EGBznjNxiv724kbuS5ZfXsrH2wx2SPKjt5BZnDiUrgCjdiHBk-0PkIdxw2LfZ9VlUqu_Abd-VbdRWzKbt2AYFZKH9lh24OadgIQGlxMfz8tkAS1HTsXuway_qqhAWtyneXEDszWWTvwehkUw77-o5pYrMX72Voxwyql1HjztSw1mrDGYeZ5IiU1au_xUC97saZHkRZh4_ZReJYekODih0i7Xwla8w8tDxSJKuZ2a0y-uJYVGRmNQFcZQ1bcYQth6otcBA", keys.as_slice(), &Validation::new(Algorithm::RS256)).await.is_none());
        let claims = parse_token(
            token.as_str(),
            keys.as_slice(),
            &Validation::new(Algorithm::RS256),
        )
        .await
        .unwrap();
        assert_eq!(claims.parse("sub").unwrap(), "1234567890");
        assert_eq!(claims.parse("name").unwrap(), "John Doe");
        assert_eq!(claims.parse("admin").unwrap(), "true");
        assert_eq!(claims.parse("extra.email").unwrap(), "john@doe.com");
    }
}
