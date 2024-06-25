use std::collections::HashMap;

use crate::utils::time::Frequency;

#[derive(Debug)]
pub struct Rules {
    pub root: Option<Quota>,
    pub tokens: HashMap<String, Quota>,
}

impl Rules {
    pub fn new(root: Option<Quota>, tokens: HashMap<String, Quota>) -> Self {
        Self { root, tokens }
    }
}

#[derive(Debug)]
pub struct Quota {
    pub total: Frequency,
    pub user: Option<Frequency>,
}
