use crate::http::Request;

use super::ctx::Id;

pub use param::{ParamRouter, ParamRouterBuilder};
pub use regex::{RegexRouter, RegexRouterBuilder};

mod param;
mod regex;

pub type RouterService = Box<dyn Router>;

pub trait Router {
    fn matches(&self, request: &Request) -> Option<Id>;
}

pub type RouterBuilderService = Box<dyn RouterBuilder>;

pub trait RouterBuilder {
    fn build(self: Box<Self>) -> (Vec<String>, RouterService);
}
