#[derive(Debug)]
pub struct Connection {
    pub addr: String,
    pub host: Option<String>,
    pub tls: bool,
}

impl Connection {
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            host: None,
            tls: false,
        }
    }

    pub fn with_host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn with_tls(mut self) -> Self {
        self.tls = true;
        self
    }
}
