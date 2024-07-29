#[derive(Debug)]
pub struct Connection {
    pub addr: String,
    pub host: String,
}

impl Connection {
    pub fn new(addr: String, host: String) -> Self {
        Self { addr, host }
    }
}
