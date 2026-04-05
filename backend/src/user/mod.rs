use aide::axum::ApiRouter;
use centaurus::backend::{middleware::rate_limiter::RateLimiter, user};

use crate::utils::UpdateMessage;

mod management;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router())
    .merge(user::router::<UpdateMessage>(rate_limiter))
}
