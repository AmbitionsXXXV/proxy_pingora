use async_trait::async_trait;
use pingora::{
    http::ResponseHeader,
    prelude::*,
    proxy::{ProxyHttp, Session},
};
use tracing::{info, warn};

pub mod conf;

pub struct SimpleProxy {}

#[async_trait]
impl ProxyHttp for SimpleProxy {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    // -- 上游选择与连接 --
    // 上游选择与连接是代理服务中非常重要的一个环节，它决定了代理服务器如何选择合适的上游服务器，并建立连接。
    // 在 pingora 中，上游选择与连接的实现主要依赖于 HttpPeer 结构体和 upstream_peer 方法。
    // 下面详细介绍这两个部分：
    //
    // 1. HttpPeer 结构体
    // HttpPeer 是 pingora 中用于表示上游服务器的结构体。它包含了上游服务器的地址、是否启用 TLS、上游服务器的名称等信息。
    // 在 SimpleProxy 中，我们使用 HttpPeer::new 方法创建一个新的 HttpPeer 实例，并指定上游服务器的地址、是否启用 TLS、上游服务器的名称等信息。
    // 例如：
    // let peer = HttpPeer::new("127.0.0.1:3000".to_string(), false, "localhost".to_string());
    //
    // 2. upstream_peer 方法
    // upstream_peer 方法是 pingora 中用于选择上游服务器的核心方法。它接收一个 Session 实例和上下文 ctx 作为参数，并返回一个 Box<HttpPeer> 实例。
    // 在 SimpleProxy 中，我们使用 upstream_peer 方法选择上游服务器，并返回一个 Box<HttpPeer> 实例。
    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let peer = HttpPeer::new("127.0.0.1:3000".to_string(), false, "localhost".to_string());
        info!("upstream_peer {}", peer.to_string());

        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut (),
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        info!("upstream_request_filter {:?}", upstream_request);
        upstream_request.insert_header("user-agent", "SimpleProxy/0.1")?;

        Ok(())
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) {
        info!("upstream_response_filter: {:?}", upstream_response);

        if let Err(e) = upstream_response.insert_header("x-simple-proxy", "v0.1") {
            warn!("failed to insert header: {}", e);
        }

        match upstream_response.remove_header("server") {
            Some(server) => {
                if let Err(e) = upstream_response.insert_header("server", server) {
                    warn!("failed to insert header: {}", e);
                }
            }
            None => {
                if let Err(e) = upstream_response.insert_header("server", "SimpleProxy/0.1") {
                    warn!("failed to insert header: {}", e);
                }
            }
        }
    }
}
