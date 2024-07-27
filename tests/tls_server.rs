#![cfg(feature = "tls")]
mod tests {
    use async_trait::async_trait;
    use essentials::debug;
    use gateway::http::response::ResponseBody;
    use gateway::{http::HeaderMapExt, tcp, ParamRouterBuilder};
    use gateway::{ReadResponse, Request, Response, WriteHalf, WriteRequest};
    use http::{header, Method, StatusCode};
    use pretty_assertions::assert_eq;
    use rcgen::{generate_simple_self_signed, CertifiedKey};
    use std::io;
    use std::net::SocketAddr;
    use std::{env, sync::Arc};
    use testing_utils::macros as utils;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::task::JoinHandle;
    use tokio_rustls::rustls::pki_types::{PrivateKeyDer, ServerName};
    use tokio_rustls::rustls::{ClientConfig, RootCertStore, ServerConfig};
    use tokio_rustls::{TlsAcceptor, TlsConnector};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

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

    struct Context {
        app: u16,
        domain: String,
        connector: TlsConnector,
        _origin_server: MockServer,
        _app_server: JoinHandle<()>,
    }

    #[derive(Debug)]
    struct Body(pub String);

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

    async fn run_request(request: Request, ctx: &Context) -> Response {
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

    async fn before_each() -> Context {
        if env::var("CI").is_err() {
            env::set_var("RUST_LOG", "debug");
            env::set_var("RUST_BACKTRACE", "0");
            env::set_var("APP_ENV", "d");
            essentials::install();
        }
        let domain = "hello.world.example".to_string();
        let subject_alt_names = vec![domain.clone()];
        let CertifiedKey { cert, key_pair } =
            generate_simple_self_signed(subject_alt_names).unwrap();

        let listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let mock_addr = listener.local_addr().unwrap().to_string();
        let mock_server = MockServer::builder().listener(listener).start().await;
        Mock::given(method("GET"))
            .and(path("/hello"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello, world!"))
            .mount(&mock_server)
            .await;
        let ports = testing_utils::get_random_ports(3);
        let server = gateway::builder(
            tcp::Builder::new()
                .add_peer("app", tcp::config::Connection::new(mock_addr))
                .build(),
            |request| {
                request
                    .header(header::HOST)
                    .and_then(|value| value.to_str().ok())
                    .map(|x| x.to_string())
            },
        )
        .with_app_port(ports[0])
        .with_tls(
            ports[1],
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
        .with_health_check_port(ports[2])
        .register_peer(
            "app".to_string(),
            ParamRouterBuilder::new().add_route(
                Method::GET,
                "/hello".to_string(),
                "hello".to_string(),
            ),
        )
        .build()
        .await
        .unwrap();
        let server_thread = tokio::spawn(server.run());
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.add(cert.der().clone()).unwrap();

        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        wait_for_server(ports[2]).await;
        Context {
            app: ports[1],
            domain,
            connector,
            _app_server: server_thread,
            _origin_server: mock_server,
        }
    }

    async fn after_each(_ctx: ()) {}

    async fn wait_for_server(health_check: u16) {
        use testing_utils::surf;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            if let Ok(response) = surf::get(format!("http://127.0.0.1:{}", health_check)).await {
                if response.status() == 200 {
                    break;
                }
            }
            interval.tick().await;
        }
    }
}
