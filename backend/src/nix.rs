use axum::{
  Router,
  body::Body,
  extract::{FromRequestParts, Path},
  routing::{get, head},
};
use centaurus::{bail, db::init::Connection, error::Result};
use http::{HeaderMap, HeaderName, StatusCode, header};
use serde::Deserialize;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
  cache::storage::FileStorage,
  db::{DBTrait, nar::NarInfoData},
};

/// https://fzakaria.github.io/nix-http-binary-cache-api-spec/#/default
pub fn router() -> Router {
  Router::new()
    .route("/{uuid}/nix-cache-info", get(nix_cache_info))
    .route("/{uuid}/{path}", head(head_nar_info))
    .route("/{uuid}/{path}", get(nar_info))
    .route("/{uuid}/nar/{hash}", get(nar))
}

#[derive(FromRequestParts, Deserialize)]
#[from_request(via(Path))]
struct CachePath {
  uuid: Uuid,
}

async fn nix_cache_info(db: Connection, path: CachePath) -> Result<(HeaderMap, String)> {
  let Some(cache) = db.cache().by_id(path.uuid).await? else {
    bail!(NOT_FOUND, "Cache not found");
  };

  let mut headers = HeaderMap::new();
  headers.insert("Content-Type", "text/x-nix-cache-info".parse().unwrap());

  Ok((
    headers,
    format!(
      "StoreDir: /nix/store
WantMassQuery: 1
Priority: {}",
      cache.priority
    ),
  ))
}

#[derive(FromRequestParts, Deserialize)]
#[from_request(via(Path))]
struct NarInfoPath {
  uuid: Uuid,
  path: String,
}

async fn get_data(db: &Connection, path: NarInfoPath) -> Result<NarInfoData> {
  let Some(hash) = path.path.strip_suffix(".narinfo") else {
    bail!(NOT_FOUND, "Invalid narinfo path");
  };

  let Some(data) = db.nar().nar_info_data(path.uuid, hash).await? else {
    bail!(NOT_FOUND, "Narinfo not found");
  };

  Ok(data)
}

async fn head_nar_info(db: Connection, path: NarInfoPath) -> Result<HeaderMap> {
  get_data(&db, path).await?;

  let mut headers = HeaderMap::new();
  headers.insert("Content-Type", "text/x-nix-narinfo".parse().unwrap());

  Ok(headers)
}

async fn nar_info(db: Connection, path: NarInfoPath) -> Result<(HeaderMap, String)> {
  let data = get_data(&db, path).await?;
  let references = db.nar().nar_info_references(data.id).await?;

  let mut headers = HeaderMap::new();
  headers.insert("Content-Type", "text/x-nix-narinfo".parse().unwrap());

  Ok((
    headers,
    format!(
      "StorePath: /nix/store/{}
URL: nar/{}.nar.{}
Compression: {}
FileHash: sha256:{}
FileSize: {}
NarHash: sha256:{}
NarSize: {}
References: {}{}
Sig: {}",
      data.store_path,
      data.hash,
      data.compression,
      data.compression,
      data.hash,
      data.size,
      data.nar_hash,
      data.nar_size,
      references.join(" "),
      data
        .deriver
        .map(|d| format!("\nDeriver: {}", d))
        .unwrap_or_default(),
      data.signature
    ),
  ))
}

#[derive(FromRequestParts, Deserialize)]
#[from_request(via(Path))]
struct NarPath {
  uuid: Uuid,
  hash: String,
}

async fn nar(
  db: Connection,
  path: NarPath,
  storage: FileStorage,
) -> Result<(StatusCode, [(HeaderName, String); 3], Body)> {
  // parse hash as <hash>.nar.<compression>
  let Some((hash, compression)) = path.hash.split_once(".nar.") else {
    bail!(NOT_FOUND, "Invalid nar path");
  };

  let Some((nar_id, file_size)) = db.nar().get_nar(path.uuid, hash, compression).await? else {
    bail!(NOT_FOUND, "Nar not found");
  };
  tracing::info!("Serving nar {} for cache {}", nar_id, path.uuid);

  let stream = storage.get_file(nar_id).await?;

  let body = Body::from_stream(ReaderStream::new(stream));

  let headers = [
    (header::ACCEPT_RANGES, "bytes".into()),
    (header::CONTENT_TYPE, "application/x-nix-nar".into()),
    (header::CONTENT_LENGTH, file_size.to_string()),
  ];

  Ok((StatusCode::OK, headers, body))
}
