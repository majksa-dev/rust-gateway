use http::Method;

use crate::{gateway::router::RouterBuilder, ParamRouter, RouterService};

type Route = (Method, String, String);

#[derive(Debug, Default)]
pub struct ParamRouterBuilder {
    routes: Vec<Route>,
}

impl ParamRouterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_route(mut self, method: Method, path: String, app_id: String) -> Self {
        self.routes.push((method, path, app_id));
        self
    }
}

impl RouterBuilder for ParamRouterBuilder {
    fn build(self: Box<Self>) -> (Vec<String>, RouterService) {
        (
            self.routes.iter().map(|(_, _, id)| id.clone()).collect(),
            self.routes
                .into_iter()
                .enumerate()
                .map(|(id, (method, path, _))| (method, path, id))
                .collect::<Box<ParamRouter>>(),
        )
    }
}

impl From<Vec<Route>> for ParamRouterBuilder {
    fn from(routes: Vec<Route>) -> Self {
        Self { routes }
    }
}

impl FromIterator<Route> for ParamRouterBuilder {
    fn from_iter<T: IntoIterator<Item = Route>>(iter: T) -> Self {
        Self {
            routes: iter.into_iter().collect(),
        }
    }
}
