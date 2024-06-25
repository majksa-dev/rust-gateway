use http::Method;
use regex::Regex;

use crate::{gateway::router::RouterBuilder, RegexRouter, RouterService};

#[derive(Debug, Default)]
pub struct RegexRouterBuilder {
    routes: Vec<(Method, Regex, String)>,
}

impl RegexRouterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_route(&mut self, method: Method, regex: Regex, app_id: String) {
        self.routes.push((method, regex, app_id));
    }
}

impl RouterBuilder for RegexRouterBuilder {
    fn build(self: Box<Self>) -> (Vec<String>, RouterService) {
        (
            self.routes.iter().map(|(_, _, id)| id.clone()).collect(),
            self.routes
                .into_iter()
                .enumerate()
                .map(|(id, (method, regex, _))| (method, regex, id))
                .collect::<Box<RegexRouter>>(),
        )
    }
}

impl From<Vec<(Method, Regex, String)>> for RegexRouterBuilder {
    fn from(routes: Vec<(Method, Regex, String)>) -> Self {
        Self { routes }
    }
}
