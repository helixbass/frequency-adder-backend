// https://github.com/graphql-rust/juniper/blob/juniper_axum-v0.3.0/juniper_axum/examples/simple.rs

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use juniper::{graphql_object, EmptySubscription, RootNode};
use juniper_axum::{extract::JuniperRequest, graphiql, response::JuniperResponse};
use tempdir::TempDir;
use tokio::{fs, net::TcpListener, task::spawn_blocking};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use uuid::Uuid;

mod wav;
mod wav_file;

#[derive(Clone)]
struct Context {
    pub temp_dir: Arc<TempDir>,
}

impl Context {
    fn new(temp_dir: Arc<TempDir>) -> Self {
        Self { temp_dir }
    }
}

impl juniper::Context for Context {}

#[derive(Copy, Clone, Debug)]
struct Query;

#[graphql_object]
#[graphql(context = Context)]
impl Query {
    async fn wav_file_url(context: &Context, uuid: Uuid) -> Option<String> {
        let wav_file_fs_path = wav_file::fs_path(&context.temp_dir, uuid);

        fs::try_exists(&wav_file_fs_path)
            .await
            .unwrap()
            .then(|| wav_file::url_path(uuid))
    }
}

#[derive(Copy, Clone, Debug)]
struct Mutation;

#[graphql_object]
#[graphql(context = Context)]
impl Mutation {
    async fn create_wav_file(context: &Context, frequency: f64) -> Uuid {
        let uuid = Uuid::new_v4();

        spawn_blocking({
            let temp_dir = context.temp_dir.clone();
            move || {
                wav::write_wav_file(frequency as f32, &temp_dir, uuid);
            }
        });

        uuid
    }
}

type Schema = RootNode<Query, Mutation, EmptySubscription<Context>>;

async fn custom_graphql(
    Extension(schema): Extension<Arc<Schema>>,
    Extension(context): Extension<Context>,
    JuniperRequest(request): JuniperRequest,
) -> JuniperResponse {
    JuniperResponse(request.execute(&*schema, &context).await)
}

#[tokio::main]
async fn main() {
    let schema = Schema::new(Query, Mutation, EmptySubscription::new());

    let temp_dir = Arc::new(TempDir::new("wav_files").expect("couldn't create temp dir"));

    let context = Context::new(temp_dir.clone());

    let app = Router::new()
        .nest_service(wav_file::URL_PATH_PREFIX, ServeDir::new(&*temp_dir))
        .route(
            "/graphql",
            on(MethodFilter::GET.or(MethodFilter::POST), custom_graphql),
        )
        .route("/graphiql", get(graphiql("/graphql", None)))
        .layer(Extension(Arc::new(schema)))
        .layer(Extension(context))
        .layer(
            CorsLayer::new()
                // TODO: restrict this or whatever
                .allow_origin(Any)
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve()`: {e}"));
}
