mod helper;

#[cfg(feature = "tls")]
mod tests {
    use essentials::debug;
    use gateway::{http::HeaderMapExt, Request};
    use helper::*;
    use http::{header, Method, StatusCode};
    use pretty_assertions::assert_eq;
    use testing_utils::macros as utils;

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_succeed(ctx: Context) {
        let mut request = Request::new("/hello".to_string(), Method::GET);
        request.insert_header(header::HOST, "app");
        request.insert_header(header::CONTENT_LENGTH, "0");
        let response = run_request(request, &ctx).await;
        debug!("{:?}", response);
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.body().unwrap().read_all(0).await.unwrap(),
            "Hello, world!"
        );
    }

    mod helper {
        use async_trait::async_trait;
        use essentials::debug;
        use gateway::{
            http::{response::ResponseBody, HeaderMapExt},
            ReadResponse, Request, Response, WriteHalf, WriteRequest,
        };
        use rcgen::{generate_simple_self_signed, CertifiedKey};
        use std::{io, sync::Arc};
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpStream,
        };
        use tokio_rustls::{
            rustls::{
                pki_types::{PrivateKeyDer, ServerName},
                ClientConfig, RootCertStore, ServerConfig,
            },
            TlsAcceptor, TlsConnector,
        };

        #[derive(Debug)]
        pub struct Body(pub String);

        #[async_trait]
        impl ResponseBody for Body {
            async fn read_all(self: Box<Self>, _len: usize) -> io::Result<String> {
                Ok(self.0)
            }

            async fn copy_to<'a>(
                &mut self,
                writer: &'a mut WriteHalf,
                _length: Option<usize>,
            ) -> io::Result<()> {
                writer.write_all(self.0.as_bytes()).await?;
                Ok(())
            }
        }

        pub async fn run_request(request: Request, ctx: &Context) -> Response {
            let stream = TcpStream::connect(&format!("127.0.0.1:{}", ctx.app))
                .await
                .unwrap();
            let domain = ServerName::try_from(ctx.domain.as_str())
                .unwrap()
                .to_owned();
            let mut stream = ctx.connector.connect(domain, stream).await.unwrap();
            stream.write_request(&request).await.unwrap();
            stream.flush().await.unwrap();
            // stream.shutdown().await.unwrap();
            let (mut response, remains) = stream.read_response().await.unwrap();
            debug!(?response, "read response");
            let mut body = String::from_utf8(remains.to_vec()).unwrap();
            let length = response
                .get_content_length()
                .unwrap()
                .saturating_sub(remains.len());
            if length > 0 {
                let mut buf = vec![0; length];
                stream.read_exact(&mut buf).await.unwrap();
                body.push_str(&String::from_utf8(buf).unwrap());
            }
            debug!(?response, "read response body");
            response.set_body(Body(body));
            response
        }

        pub struct Context {
            _context: crate::helper::Context,
            pub app: u16,
            pub domain: String,
            connector: TlsConnector,
        }

        pub async fn before_each() -> Context {
            let domain = "hello.world.example".to_string();
            let subject_alt_names = vec![domain.clone()];
            let CertifiedKey { cert, key_pair } =
                generate_simple_self_signed(subject_alt_names).unwrap();
            let mut root_cert_store = RootCertStore::empty();
            root_cert_store.add(cert.der().clone()).unwrap();

            let config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();
            let connector = TlsConnector::from(Arc::new(config));
            let (context, ports) = crate::helper::setup_with_ports(1, |server_builder, ports| {
                server_builder.with_tls(
                    ports[0],
                    TlsAcceptor::from(Arc::new(
                        ServerConfig::builder()
                            .with_no_client_auth()
                            .with_single_cert(
                                vec![cert.der().clone()],
                                PrivateKeyDer::try_from(key_pair.serialize_der()).unwrap(),
                            )
                            .unwrap(),
                    )),
                )
            })
            .await;
            Context {
                _context: context,
                app: ports[0],
                domain,
                connector,
            }
        }

        pub async fn after_each(_ctx: ()) {}
    }
}
