use std::pin::Pin;
use std::task::Poll;

use futures::Future;

use crate::{client::Body, Api};

use super::Proxy;

/// Proxies a request by prepending it with the URL of the `[Proxy]` resource
/// and sending it through the `[crate::Client]`
///
/// Note, this is not a proxy server. It does not listen on a port, it just
/// modifies incoming requests and uses the [`crate::Client`] to send them.
#[derive(Clone)]
pub struct Proxier<K> {
    api: Api<K>,
    name: String,
    port: Option<String>,
}

impl<K> Proxier<K> where K: Proxy {
    /// Creates a new proxier for the `[Proxy]` `K` with `name` and port `port`
    pub fn new(api: Api<K>, name: &str, port: Option<&str>) -> Self {
        Self { api, name: name.to_string(), port: port.map(|p| p.to_string()) }
    }
}

impl<B, K> tower::Service<http::Request<B>> for Proxier<K>
where
    B: From<Body> + 'static,
    K: Proxy + Clone,
    Body: From<B>,
{
    type Response = http::Response<B>;
    type Error = crate::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        let api = self.api.clone();
        let req = api.proxied(&self.name, self.port.as_deref(), req);

        Box::pin(async move {
            let resp = api.client.send(req?.map(Body::from)).await?;

            Ok(resp.map(B::from))
        })
    }
}
