use std::str::FromStr;

use async_compression::tokio::bufread::ZstdEncoder;
use harmonia_store_core::store_path::StorePath;
use harmonia_store_remote::{DaemonClient, DaemonStore};
use tokio::{fs::File, io};

#[tokio::main]
async fn main() {
  println!("Hello, world!");

  let path = "/nix/store/hlxw2q9qansq7bn52xvlb5badw3z1v8s-coreutils-9.10";
  let base_path = "/nix/store/";

  let mut conn = DaemonClient::builder().connect_daemon().await.unwrap();
  let path = StorePath::from_str(path.strip_prefix(base_path).unwrap()).unwrap();
  let info = conn.query_path_info(&path).await.unwrap().unwrap();

  println!("Daemon info: {:?}", info);

  let nar = conn.nar_from_path(&path).await.unwrap();
  let mut encoder = ZstdEncoder::new(nar);

  let mut file = File::create("output.nar.zst").await.unwrap();
  io::copy(&mut encoder, &mut file).await.unwrap();
}
