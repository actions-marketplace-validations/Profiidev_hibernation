use centaurus::db::init::Connection;

use crate::db::cache::CacheTable;
use crate::db::group::GroupTable;
use crate::db::nar::NarTable;
use crate::db::setup::SetupTable;
use crate::db::token::TokenTable;
use crate::db::user::UserTable;

pub mod cache;
pub mod group;
pub mod nar;
pub mod setup;
pub mod token;
pub mod user;

pub trait DBTrait {
  fn setup(&self) -> SetupTable<'_>;
  fn group(&self) -> GroupTable<'_>;
  fn user(&self) -> UserTable<'_>;
  fn token(&self) -> TokenTable<'_>;
  fn cache(&self) -> CacheTable<'_>;
  fn nar(&self) -> NarTable<'_>;
}

impl DBTrait for Connection {
  fn setup(&self) -> SetupTable<'_> {
    SetupTable::new(self)
  }

  fn group(&self) -> GroupTable<'_> {
    GroupTable::new(self)
  }

  fn user(&self) -> UserTable<'_> {
    UserTable::new(self)
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
