use entity::{cache, nar_info};
use sea_orm::{ActiveValue::Set, FromQueryResult, Iterable, JoinType, QuerySelect, prelude::*};
use serde::Serialize;

#[derive(Serialize, FromQueryResult)]
pub struct CacheInfo {
  #[serde(rename = "uuid")]
  id: Uuid,
  name: String,
  #[serde(serialize_with = "serialize_i64_or_zero")]
  size: Option<i64>,
  quota: i64,
  public: bool,
}

#[derive(Serialize, FromQueryResult)]
pub struct CacheDetails {
  #[serde(rename = "uuid")]
  id: Uuid,
  name: String,
  #[serde(serialize_with = "serialize_i64_or_zero")]
  size: Option<i64>,
  quota: i64,
  public: bool,
  #[serde(rename = "sig_key")]
  public_signing_key: String,
  priority: i32,
}

pub struct CacheTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> CacheTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn list_caches(&self, user: Uuid) -> Result<Vec<CacheInfo>, DbErr> {
    cache::Entity::find()
      .join(JoinType::LeftJoin, cache::Relation::NarInfo.def())
      .group_by(cache::Column::Id)
      .select_only()
      .columns(cache::Column::iter())
      .column_as(nar_info::Column::FileSize.sum(), "size")
      .into_model::<CacheInfo>()
      .all(self.db)
      .await
  }

  pub async fn cache_details(&self, uuid: Uuid) -> Result<Option<CacheDetails>, DbErr> {
    cache::Entity::find_by_id(uuid)
      .join(JoinType::LeftJoin, cache::Relation::NarInfo.def())
      .group_by(cache::Column::Id)
      .select_only()
      .columns(cache::Column::iter())
      .column_as(nar_info::Column::FileSize.sum(), "size")
      .into_model::<CacheDetails>()
      .one(self.db)
      .await
  }

  pub async fn by_name(&self, name: String) -> Result<Option<cache::Model>, DbErr> {
    cache::Entity::find()
      .filter(cache::Column::Name.eq(name))
      .one(self.db)
      .await
  }

  pub async fn create_cache(
    &self,
    name: String,
    public: bool,
    quota: i64,
    sig_key: String,
  ) -> Result<Uuid, DbErr> {
    let cache = cache::ActiveModel {
      id: Set(Uuid::new_v4()),
      name: Set(name),
      public: Set(public),
      quota: Set(quota),
      public_signing_key: Set(sig_key),
      priority: Set(50),
    };
    let res = cache.insert(self.db).await?;
    Ok(res.id)
  }

  pub async fn delete_cache(&self, uuid: Uuid) -> Result<(), DbErr> {
    cache::Entity::delete_by_id(uuid).exec(self.db).await?;

    Ok(())
  }
}

fn serialize_i64_or_zero<S>(num: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let num = num.unwrap_or(0);
  serializer.serialize_i64(num)
}
