use http::Method;

use crate::{Id, Request, Router};

type Route = (Method, String, Id);

#[derive(Debug, Default)]
pub struct ParamRouter {
    routes: Vec<Route>,
}

impl FromIterator<Route> for Box<ParamRouter> {
    fn from_iter<T: IntoIterator<Item = Route>>(routes: T) -> Self {
        Box::new(ParamRouter {
            routes: routes.into_iter().collect(),
        })
    }
}

impl Router for ParamRouter {
    fn matches(&self, request: &Request) -> Option<Id> {
        for (method, path, app_id) in &self.routes {
            if method == request.method && path.matches(&request.path) {
                return Some(*app_id);
            }
        }
        None
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
