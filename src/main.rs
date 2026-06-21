// https://github.com/graphql-rust/juniper/blob/juniper_axum-v0.3.0/juniper_axum/examples/simple.rs

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, RootNode};
use juniper_axum::{extract::JuniperRequest, graphiql, response::JuniperResponse};
use tempdir::TempDir;
use tokio::{fs, net::TcpListener};
use tower_http::services::ServeDir;
use url::Url;
use uuid::Uuid;

const WAV_FILES_URL_PATH_PREFIX: &'static str = "/wav_files";

fn wav_file_name(uuid: Uuid) -> String {
    format!("{}.wav", uuid)
}

fn wav_file_fs_path(temp_dir: &TempDir, uuid: Uuid) -> PathBuf {
    temp_dir.as_ref().join(&wav_file_name(uuid))
}

fn wav_file_url_path(uuid: Uuid) -> Url {
    Url::parse(&format!(
        "{}/{}",
        WAV_FILES_URL_PATH_PREFIX,
        wav_file_name(uuid)
    ))
    .unwrap()
}

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
    async fn wav_file_url(context: &Context, uuid: Uuid) -> Option<Url> {
        let wav_file_fs_path = wav_file_fs_path(&context.temp_dir, uuid);

        fs::try_exists(&wav_file_fs_path)
            .await
            .unwrap()
            .then(|| wav_file_url_path(uuid))
    }
}

type Schema = RootNode<Query, EmptyMutation<Context>, EmptySubscription<Context>>;

async fn custom_graphql(
    Extension(schema): Extension<Arc<Schema>>,
    Extension(context): Extension<Context>,
    JuniperRequest(request): JuniperRequest,
) -> JuniperResponse {
    JuniperResponse(request.execute(&*schema, &context).await)
}

#[tokio::main]
async fn main() {
    let schema = Schema::new(Query, EmptyMutation::new(), EmptySubscription::new());

    let temp_dir = Arc::new(TempDir::new("wav_files").expect("couldn't create temp dir"));

    let context = Context::new(temp_dir.clone());

    let app = Router::new()
        .nest_service(WAV_FILES_URL_PATH_PREFIX, ServeDir::new(&*temp_dir))
        .route(
            "/graphql",
            on(MethodFilter::GET.or(MethodFilter::POST), custom_graphql),
        )
        .route("/graphiql", get(graphiql("/graphql", None)))
        .layer(Extension(Arc::new(schema)))
        .layer(Extension(context));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve()`: {e}"));
}
