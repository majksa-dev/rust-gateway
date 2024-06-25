use crate::time::Time;

#[derive(Debug)]
pub struct Endpoint {
    pub expires_in: Time,
    pub vary_headers: Vec<String>,
}

impl Endpoint {
    pub fn new(expires_in: Time, vary_headers: Vec<String>) -> Self {
        Self {
            expires_in,
            vary_headers,
        }
    }
}
