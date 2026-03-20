use axum::{
  Json, Router,
  extract::{FromRequest, FromRequestParts, Path},
  routing::{delete, get, post},
};
use centaurus::{bail, db::init::Connection, error::Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  auth::jwt_auth::JwtAuth,
  db::{
    DBTrait,
    cache::{CacheDetails, CacheInfo},
  },
  permissions::CacheCreate,
};

pub fn router() -> Router {
  Router::new()
    .route("/", get(list_caches))
    .route("/", post(create_cache))
    .route("/", delete(delete_cache))
    .route("/{uuid}", get(cache_details))
}

async fn list_caches(auth: JwtAuth, db: Connection) -> Result<Json<Vec<CacheInfo>>> {
  Ok(Json(db.cache().list_caches(auth.user_id).await?))
}

#[derive(Deserialize, FromRequestParts)]
#[from_request(via(Path))]
struct CachePath {
  uuid: Uuid,
}

async fn cache_details(
  auth: JwtAuth,
  path: CachePath,
  db: Connection,
) -> Result<Json<CacheDetails>> {
  let Some(details) = db.cache().cache_details(path.uuid, auth.user_id).await? else {
    bail!(NOT_FOUND, "Cache not found");
  };

  Ok(Json(details))
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Json))]
struct CreateCacheRequest {
  name: String,
  public: bool,
  quota: i64,
  sig_key: String,
}

#[derive(Serialize)]
struct CreateCacheResponse {
  uuid: Uuid,
}

async fn create_cache(
  _auth: JwtAuth<CacheCreate>,
  db: Connection,
  req: CreateCacheRequest,
) -> Result<Json<CreateCacheResponse>> {
  if db.cache().by_name(req.name.clone()).await?.is_some() {
    bail!(CONFLICT, "Cache with this name already exists");
  }

  let uuid = db
    .cache()
    .create_cache(req.name, req.public, req.quota, req.sig_key)
    .await?;

  Ok(Json(CreateCacheResponse { uuid }))
}

async fn delete_cache(auth: JwtAuth, db: Connection) -> Result<()> {
  db.cache().delete_cache(auth.user_id).await?;
  Ok(())
}
