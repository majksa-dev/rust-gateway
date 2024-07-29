#[derive(Debug)]
pub struct Connection {
    pub addr: String,
    pub host: Option<String>,
}

impl Connection {
    pub fn new(addr: String) -> Self {
        Self { addr, host: None }
    }

    pub fn with_host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }
}
