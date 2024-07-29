use crate::{Router, RouterBuilder};

use super::router::AnyRouter;

pub struct AnyRouterBuilder;

impl RouterBuilder for AnyRouterBuilder {
    fn build(self: Box<Self>) -> (Vec<String>, Box<dyn Router>) {
        (vec![String::new()], Box::new(AnyRouter))
    }
}
