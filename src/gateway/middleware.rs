use async_trait::async_trait;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    proxy::Session,
    Result,
};

/// Middleware is a trait that can be used to filter and modify requests and responses.
/// It can be used to implement custom logic for handling requests and responses.
/// Register a middleware using the [crate::server::app::ServerBuilder::register_middleware] method.
#[async_trait]
pub trait Middleware {
    /// Filter the request before sending it to the upstream server.
    /// If the function returns false, the request will be dropped
    /// and instead a response will be sent to the client.
    /// If the function returns true, the request will be sent to the upstream server.
    async fn filter(&self, _session: &mut Session) -> Result<bool> {
        Ok(true)
    }

    /// Modify the request before sending it to the upstream server.
    async fn modify_request(
        &self,
        _session: &mut Session,
        _request: &mut RequestHeader,
    ) -> Result<()> {
        Ok(())
    }

    /// Modify the response before returning it to the client.
    async fn modify_response(
        &self,
        _session: &mut Session,
        _response: &mut ResponseHeader,
    ) -> Result<()> {
        Ok(())
    }
}