use axum::{Router, routing::get};
use centaurus::error::Result;

use crate::auth::cli_auth::CliAuth;

mod management;

pub fn router() -> Router {
  Router::new()
    .nest("/management", management::router())
    .route("/test", get(test))
}

async fn test(auth: CliAuth) -> Result<String> {
  Ok(auth.user_id.to_string())
}
