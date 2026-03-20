use entity::{cache, cache_access, group_user, nar_info, sea_orm_active_enums::AccessType};
use sea_orm::{
  ActiveValue::Set, Condition, FromQueryResult, Iterable, JoinType, QuerySelect, prelude::*,
};
use serde::Serialize;

use crate::db::setup::SetupTable;

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

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
  GroupId,
}

pub struct CacheTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> CacheTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn list_caches(&self, user: Uuid) -> Result<Vec<CacheInfo>, DbErr> {
    let mut query = cache::Entity::find()
      .join(JoinType::LeftJoin, cache::Relation::NarInfo.def())
      .join(JoinType::LeftJoin, cache::Relation::CacheAccess.def());

    query = apply_user_filter(query, self.db, user, AccessType::View).await?;

    query
      .group_by(cache::Column::Id)
      .select_only()
      .columns(cache::Column::iter())
      .column_as(nar_info::Column::FileSize.sum(), "size")
      .into_model::<CacheInfo>()
      .all(self.db)
      .await
  }

  pub async fn cache_details(&self, uuid: Uuid, user: Uuid) -> Result<Option<CacheDetails>, DbErr> {
    let mut query = cache::Entity::find_by_id(uuid)
      .join(JoinType::LeftJoin, cache::Relation::NarInfo.def())
      .join(JoinType::LeftJoin, cache::Relation::CacheAccess.def());

    query = apply_user_filter(query, self.db, user, AccessType::View).await?;

    query
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

  pub async fn by_id_filtered(
    &self,
    uuid: Uuid,
    user: Uuid,
    access_type: AccessType,
  ) -> Result<Option<cache::Model>, DbErr> {
    let mut query =
      cache::Entity::find_by_id(uuid).join(JoinType::LeftJoin, cache::Relation::CacheAccess.def());

    query = apply_user_filter(query, self.db, user, access_type).await?;

    query.one(self.db).await
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
    if self
      .by_id_filtered(uuid, Uuid::nil(), AccessType::Edit)
      .await?
      .is_none()
    {
      return Err(DbErr::Custom(
        "Cache not found or insufficient permissions".to_string(),
      ));
    }

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

async fn apply_user_filter<Q: QueryFilter>(
  mut query: Q,
  db: &DatabaseConnection,
  user: Uuid,
  access_type: AccessType,
) -> Result<Q, DbErr> {
  let group_ids = group_user::Entity::find()
    .filter(group_user::Column::UserId.eq(user))
    .select_only()
    .column_as(group_user::Column::GroupId, QueryAs::GroupId)
    .into_values::<Uuid, QueryAs>()
    .all(db)
    .await?;

  // hash all access
  let admin_group = SetupTable::new(db).get_admin_group_id().await?;
  if !admin_group.is_some_and(|id| group_ids.contains(&id)) {
    let mut user_cond = cache_access::Column::UserId.eq(user);
    let mut group_cond = cache_access::Column::GroupId.is_in(group_ids);

    // if only read access is requested, also include entries with edit access
    if access_type == AccessType::Edit {
      user_cond = user_cond.and(cache_access::Column::AccessType.eq(AccessType::Edit));
      group_cond = group_cond.and(cache_access::Column::AccessType.eq(AccessType::Edit));
    }

    query = query.filter(
      Condition::any()
        .add(user_cond)
        .add(group_cond)
        .add(cache::Column::Public.eq(true)),
    );
  };

  Ok(query)
}
