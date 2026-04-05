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

impl websocket::state::UpdateMessage for UpdateMessage {
  fn settings() -> Self {
    Self::Settings
  }

  fn user(uuid: Uuid) -> Self {
    Self::User { uuid }
  }

  fn group(uuid: Uuid) -> Self {
    Self::Group { uuid }
  }

  fn user_permissions() -> Self {
    Self::UserPermissions
  }
}

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
