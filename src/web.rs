use axum::routing::get;
use axum::extract::State;
pub async fn start_axum(indexer: std::sync::Arc<crate::penumbra::PenumbraIndexer>, bind_addr: &str) -> crate::errors::IndexerResult<()>{
    let app = axum::Router::new()
        .route("/", get(get_blocks))
        .with_state(indexer);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

use axum::http::StatusCode;
impl IntoResponse for crate::errors::ErrorWrapper {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            // TODO more error type handling
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        log::error!("{}", self);
        (status, message).into_response()
    }
}

use axum::response::IntoResponse;
use crate::errors::IndexerResult;
async fn get_blocks(State(pen): State<std::sync::Arc<crate::penumbra::PenumbraIndexer>>) -> IndexerResult<impl IntoResponse> {
    let latest = pen.get_latest_block().await;
    let last_ten = latest-9..latest+1;
    let blocks = pen.fetch_blocks_db(last_ten.collect()).await?;
    let json = serde_json::value::to_value(blocks)?;
    Ok(axum::Json(json))
}
