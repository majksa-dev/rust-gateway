use crate::{Id, Request, Router};

#[derive(Debug, Default)]
pub struct AnyRouter;

impl Router for AnyRouter {
    fn matches(&self, _request: &Request) -> Option<Id> {
        Some(0)
    }
}
