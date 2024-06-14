use http::Method;
use regex::Regex;

use crate::http::Request;

pub type RouterService = Box<dyn Router>;

pub trait Router {
    fn matches(&self, request: &Request) -> Option<&String>;
}

#[derive(Debug, Default)]
pub struct RegexRouter {
    routes: Vec<(Method, Regex, String)>,
}

impl RegexRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_route(&mut self, method: Method, regex: Regex, app_id: String) {
        self.routes.push((method, regex, app_id));
    }
}

impl From<RegexRouter> for RouterService {
    fn from(router: RegexRouter) -> Self {
        Box::new(router)
    }
}

impl From<Vec<(Method, Regex, String)>> for RegexRouter {
    fn from(routes: Vec<(Method, Regex, String)>) -> Self {
        Self { routes }
    }
}

impl Router for RegexRouter {
    fn matches(&self, request: &Request) -> Option<&String> {
        for (method, regex, app_id) in &self.routes {
            if method == request.method && regex.is_match(&request.path) {
                return Some(app_id);
            }
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct ParamRouter {
    routes: Vec<(Method, String, String)>,
}

impl ParamRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_route(mut self, method: Method, path: String, app_id: String) -> Self {
        self.routes.push((method, path, app_id));
        self
    }
}

impl From<ParamRouter> for RouterService {
    fn from(router: ParamRouter) -> Self {
        Box::new(router)
    }
}

impl From<Vec<(Method, String, String)>> for ParamRouter {
    fn from(routes: Vec<(Method, String, String)>) -> Self {
        Self { routes }
    }
}

pub trait ParamRouteMatcher {
    fn matches(&self, path: &str) -> bool;
}

impl ParamRouteMatcher for String {
    fn matches(&self, path: &str) -> bool {
        let mut matcher_it = self.chars().peekable();
        let mut req_it = path.chars().peekable();
        while let Some(c) = matcher_it.next() {
            if c == ':' {
                while let Some(c) = matcher_it.peek() {
                    if *c == '/' {
                        break;
                    }
                    matcher_it.next();
                }
                if req_it.peek().is_none() {
                    return false;
                }
                while let Some(c) = req_it.peek() {
                    if *c == '/' {
                        break;
                    }
                    req_it.next();
                }
            } else if let Some(req_c) = req_it.next() {
                if c != req_c {
                    return false;
                }
            }
        }
        req_it.peek().is_none()
    }
}

impl Router for ParamRouter {
    fn matches(&self, request: &Request) -> Option<&String> {
        for (method, path, app_id) in &self.routes {
            if method != request.method {
                continue;
            }
            if path.matches(&request.path) {
                return Some(app_id);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!("/".to_string().matches("/hello/world"), false);
        assert_eq!("/hello/:name".to_string().matches("/hello/world"), true);
        assert_eq!("/hello/:name".to_string().matches("/hello/me"), true);
        assert_eq!("/hello/:name".to_string().matches("/hello"), false);
        assert_eq!("/hello/:name".to_string().matches("/hello/"), false);
        assert_eq!("/hello/:name".to_string().matches("/hello/world/me"), false);
        assert_eq!(
            "/hello/:name/me".to_string().matches("/hello/world/me"),
            true
        );
    }
}
