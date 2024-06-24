use std::collections::HashSet;

#[derive(Debug)]
pub enum Enable {
    All,
    Endpoints(HashSet<String>),
}

impl Enable {
    pub fn is_enabled(&self, endpoint: &String) -> bool {
        match self {
            Self::All => true,
            Self::Endpoints(endpoints) => endpoints.contains(endpoint),
        }
    }
}
