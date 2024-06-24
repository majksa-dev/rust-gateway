use anyhow::Result;
use jsonwebtoken::jwk::JwkSet;

pub async fn fetch_keys(keys_url: reqwest::Url) -> Result<JwkSet> {
    let set: JwkSet = reqwest::get(keys_url).await?.json().await?;
    Ok(set)
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;

    use super::*;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_fetch_keys() {
        let listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let origin = listener.local_addr().unwrap().port();
        let mock_server = MockServer::builder().listener(listener).start().await;
        Mock::given(method("GET"))
            .and(path("/.well-known/jwks"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({
                        "keys": [
                            {
                                "e": "AQAB",
                                "kid": "3d580f0af7ace698a0cee7f230bca594dc6dbb55",
                                "kty": "RSA",
                                "n": "rhgQZT3t9MgNBv9_4qE58CLCbDfEaRd9HgPd_Zmjg1TIYjHh1UgMPVeVekyU2JiuUZPbnlEbv8WUsxyNNQJfATvfMbXaUcrePSdW32zIaMOeTbn0VXZ3tqx5IyiP0IfJt-kT9MilGAkeJn8me7x5_uNGOpiPCWQaxFxTikVUtGO5AbGh2PTULzKjVjZWwQrPB1fqEe6Ar6Im-3RcZ-zOd3N2ThgQEzLLRe4RE6bSvBQUuxX9o_AkY0SCVZZB2VhjQYBN3EUFmKsD46rrneBn64Vduy3jWtBYXA1avDRCl0Y8yQEBOrtgikEz_hog4O4EKP5mAVSf8Iyfl_RMdxrOAQ",
                                "alg": "RS256",
                                "use": "sig"
                            }
                        ]
                    })),
            )
            .mount(&mock_server)
            .await;
        let keys_url =
            reqwest::Url::parse(format!("http://localhost:{origin}/.well-known/jwks").as_str())
                .unwrap();
        let keys = fetch_keys(keys_url).await.unwrap();
        let key = keys.keys[0].clone();
        assert_eq!(
            key.common.key_id.unwrap().as_str(),
            "3d580f0af7ace698a0cee7f230bca594dc6dbb55"
        );
        assert_eq!(
            key.common.key_algorithm.unwrap(),
            jsonwebtoken::jwk::KeyAlgorithm::RS256
        );
        assert_eq!(
            key.common.public_key_use.unwrap(),
            jsonwebtoken::jwk::PublicKeyUse::Signature
        );
    }
}
