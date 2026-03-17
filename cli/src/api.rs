use centaurus::eyre::Result;
use url::Url;

pub struct ApiClient {
  token: String,
  url: Url,
}

impl ApiClient {
  pub fn new(token: String, url: Url) -> Self {
    Self { token, url }
  }

  pub async fn request_token(url: Url, code: &str) -> Result<String> {
    let url = url.join(&format!("/api/cli?code={}", code))?;
    Ok(reqwest::get(url).await?.error_for_status()?.text().await?)
  }
}
