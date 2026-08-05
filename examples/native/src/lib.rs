use axum::{routing::{get, post}, Router};
use flame_macro::flame;
use std::mem;
use std::net::SocketAddr;

pub struct FlameServer {
    router: Router,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub body: String,
}

// Module-level function
/// Initialize a new FlameServer instance.
pub fn init() -> FlameServer {
    FlameServer {
        router: Router::new(),
    }
}

impl FlameServer {
    pub fn get<H, T>(&mut self, path: &'static str, handler: H)
    where
        H: axum::handler::Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        let router = mem::take(&mut self.router);
        self.router = router.route(path, get(handler));
    }

    pub fn post<H, T>(&mut self, path: &'static str, handler: H)
    where
        H: axum::handler::Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        let router = mem::take(&mut self.router);
        self.router = router.route(path, post(handler));
    }

    pub fn router(&mut self) -> Router {
        mem::take(&mut self.router)
    }

    #[flame(daemon)]
    pub async fn listen(self, port: u16) -> std::io::Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        let listener = tokio::net::TcpListener::bind(addr).await?;

        axum::serve(listener, self.router)
            .await
            .map_err(std::io::Error::other)
    }
}
