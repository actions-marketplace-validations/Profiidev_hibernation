use centaurus::db::init::Connection;

use crate::db::cache::CacheTable;
use crate::db::group_ext::GroupTableExt;
use crate::db::nar::NarTable;
use crate::db::token::TokenTable;
use crate::db::user_ext::UserTableExt;

pub mod cache;
pub mod group_ext;
pub mod nar;
pub mod token;
pub mod user_ext;

pub trait DBTrait {
  fn group_ext(&self) -> GroupTableExt<'_>;
  fn user_ext(&self) -> UserTableExt<'_>;
  fn token(&self) -> TokenTable<'_>;
  fn cache(&self) -> CacheTable<'_>;
  fn nar(&self) -> NarTable<'_>;
}

impl DBTrait for Connection {
  fn group_ext(&self) -> GroupTableExt<'_> {
    GroupTableExt::new(self)
  }

  fn user_ext(&self) -> UserTableExt<'_> {
    UserTableExt::new(self)
  }

  fn token(&self) -> TokenTable<'_> {
    TokenTable::new(self)
  }

  fn cache(&self) -> CacheTable<'_> {
    CacheTable::new(self)
  }

  fn nar(&self) -> NarTable<'_> {
    NarTable::new(self)
  }
}
