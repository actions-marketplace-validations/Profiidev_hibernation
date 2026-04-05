use centaurus::{
  backend::{
    auth::permission::{self, Permission},
    websocket,
  },
  permission,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Updater = websocket::state::Updater<UpdateMessage>;

impl websocket::state::UpdateMessage for UpdateMessage {}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(tag = "type")]
pub enum UpdateMessage {
  Settings,
  User { uuid: Uuid },
  UserPermissions,
  Group { uuid: Uuid },
  Token { uuid: Uuid },
  Cache { uuid: Uuid },
}

pub fn permissions() -> Vec<&'static str> {
  let mut perms = permission::permissions();
  perms.extend_from_slice(&[CacheCreate::name(), CacheView::name(), CacheEdit::name()]);
  perms
}

// Caches
permission!(CacheCreate, "cache:create");
permission!(CacheView, "cache:view");
permission!(CacheEdit, "cache:edit");
