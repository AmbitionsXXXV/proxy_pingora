use pingora::prelude::*;
use proxy_pingora::SimpleProxy;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let mut server = Server::new(None)?;
    server.bootstrap();

    let sp = SimpleProxy {};
    let mut proxy = http_proxy_service(&server.configuration, sp);
    let proxy_addr = "0.0.0.0:8080";
    proxy.add_tcp(proxy_addr);

    info!("proxy server is running on {}", proxy_addr);

    server.add_service(proxy);

    server.run_forever();
}
