use http::Method;
use regex::Regex;

use crate::{Id, Request, Router};

type Route = (Method, Regex, Id);

#[derive(Debug, Default)]
pub struct RegexRouter {
    routes: Vec<Route>,
}

impl FromIterator<Route> for Box<RegexRouter> {
    fn from_iter<T: IntoIterator<Item = Route>>(routes: T) -> Self {
        Box::new(RegexRouter {
            routes: routes.into_iter().collect(),
        })
    }
}

impl Router for RegexRouter {
    fn matches(&self, request: &Request) -> Option<Id> {
        for (method, regex, app_id) in &self.routes {
            if method == request.method && regex.is_match(&request.path) {
                return Some(*app_id);
            }
        }
        None
    }
}
