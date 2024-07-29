mod helper;

#[cfg(feature = "auth")]
mod tests {
    use essentials::debug;
    use helper::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use testing_utils::{
        macros as utils,
        surf::{self, StatusCode},
    };

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_email1(ctx: Context) {
        let token = ctx.encoding_key.generate_token(json!({
            "sub": "1234567890",
            "name": "John Doe",
            "admin": true,
            "extra": {
                "email": "john@doe.com"
            }
        }));
        let response = surf::get(format!("http://127.0.0.1:{}/email", &ctx.context.app))
            .header("Host", "app")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "john@doe.com");
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_email2(ctx: Context) {
        let token = ctx.encoding_key.generate_token(json!({
            "sub": "1234567890",
            "name": "John Doe",
            "admin": true,
            "extra": {
                "email": "john2@doe.com"
            }
        }));
        let response = surf::get(format!("http://127.0.0.1:{}/email", &ctx.context.app))
            .header("Host", "app")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "john2@doe.com");
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_401_when_no_auth_header_is_attached(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/email", &ctx.context.app))
            .header("Host", "app")
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Unauthorized);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_403_when_invalid_token_is_used(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/email", &ctx.context.app))
        .header("Host", "app")
        .header("Authorization", "Basic eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c")
        .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Unauthorized);
    }

    mod helper {
        use chrono::{Duration, Utc};
        use essentials::debug;
        use gateway::auth;
        use jsonwebtoken::{Algorithm, EncodingKey, Header};
        use serde_json::{json, Value};
        use std::net::SocketAddr;
        use wiremock::{
            matchers::{method, path},
            Mock, MockServer, ResponseTemplate,
        };

        pub struct Context {
            pub context: crate::helper::Context,
            pub encoding_key: EncodingKey,
        }

        pub async fn before_each() -> Context {
            let (jwt_server, encoding_key) = {
                let encoding_key = EncodingKey::from_rsa_pem(PRIVATE_KEY.as_bytes()).unwrap();
                let listener =
                    std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
                let server = MockServer::builder().listener(listener).start().await;
                Mock::given(method("GET"))
                    .and(path("/.well-known/jwks"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
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
                    })))
                    .mount(&server)
                    .await;
                (server, encoding_key)
            };
            debug!("Jwt server started at: {:?}", jwt_server.uri());
            let context = crate::helper::setup(|server_builder| {
                server_builder.register_middleware(
                    1,
                    auth::jwt::Builder::new()
                        .add_app_auth(
                            "app",
                            auth::jwt::config::Auth::new(
                                reqwest::Url::parse(
                                    format!("{}/.well-known/jwks", jwt_server.uri()).as_str(),
                                )
                                .unwrap(),
                                vec![auth::jwt::config::Claim {
                                    claim: "extra.email".to_string(),
                                    header: "X-Email".to_string(),
                                }],
                            ),
                        )
                        .build(),
                )
            })
            .await;
            Context {
                context,
                encoding_key,
            }
        }

        pub async fn after_each(_ctx: ()) {}

        pub trait GenerateToken {
            fn generate_token(&self, claims: Value) -> String;
        }

        impl GenerateToken for EncodingKey {
            fn generate_token(&self, mut claims: Value) -> String {
                let header = Header {
                    kid: Some("abc123".to_string()),
                    typ: Some("JWT".to_string()),
                    alg: Algorithm::RS256,
                    ..Default::default()
                };
                let in_1_day = Utc::now() + Duration::days(1);
                claims
                    .as_object_mut()
                    .unwrap()
                    .insert("iat".to_string(), in_1_day.timestamp().into());
                claims
                    .as_object_mut()
                    .unwrap()
                    .insert("exp".to_string(), in_1_day.timestamp().into());
                claims
                    .as_object_mut()
                    .unwrap()
                    .insert("nbf".to_string(), in_1_day.timestamp().into());
                jsonwebtoken::encode(&header, &claims, self).unwrap()
            }
        }

        const PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
        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
        BdfMxd9Hvq9+SlX1OOF4FW/P9FYENA==
        -----END PRIVATE KEY-----
        "#;
    }
}
