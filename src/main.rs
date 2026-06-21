// https://github.com/graphql-rust/juniper/blob/juniper_axum-v0.3.0/juniper_axum/examples/simple.rs

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, RootNode};
use juniper_axum::{graphiql, graphql};
use tokio::net::TcpListener;
use url::Url;
use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
struct Query;

#[graphql_object]
impl Query {
    fn wav_file_url(uuid: Uuid) -> Option<Url> {
        unimplemented!()
    }
}

type Schema = RootNode<Query, EmptyMutation, EmptySubscription>;

#[tokio::main]
async fn main() {
    let schema = Schema::new(Query, EmptyMutation::new(), EmptySubscription::new());

    let app = Router::new()
        .route(
            "/graphql",
            on(
                MethodFilter::GET.or(MethodFilter::POST),
                graphql::<Arc<Schema>>,
            ),
        )
        .route("/graphiql", get(graphiql("/graphql", None)))
        .layer(Extension(Arc::new(schema)));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve()`: {e}"));
}
