use centaurus::bail;
use entity::{cache, cache_access, group_user, nar_info, sea_orm_active_enums::AccessType};
use sea_orm::{
  ActiveValue::Set, Condition, FromQueryResult, Iterable, JoinType, QuerySelect, prelude::*,
};
use serde::Serialize;

use crate::{db::group::GroupTable, permissions::Permission};

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

#[derive(Serialize, FromQueryResult)]
pub struct SimpleCacheInfo {
  #[serde(rename = "uuid")]
  id: Uuid,
  name: String,
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

  pub async fn list_caches_simple(&self, user: Uuid) -> Result<Vec<SimpleCacheInfo>, DbErr> {
    let mut query =
      cache::Entity::find().join(JoinType::LeftJoin, cache::Relation::CacheAccess.def());

    query = apply_user_filter(query, self.db, user, AccessType::View).await?;

    query
      .select_only()
      .column(cache::Column::Id)
      .column(cache::Column::Name)
      .into_model::<SimpleCacheInfo>()
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
    user: Uuid,
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

    let cache_access = cache_access::ActiveModel {
      cache_id: Set(res.id),
      user_id: Set(Some(user)),
      group_id: Set(None),
      access_type: Set(AccessType::Edit),
      ..Default::default()
    };
    cache_access.insert(self.db).await?;

    Ok(res.id)
  }

  pub async fn delete_cache(&self, uuid: Uuid, user: Uuid) -> centaurus::error::Result<()> {
    if self
      .by_id_filtered(uuid, user, AccessType::Edit)
      .await?
      .is_none()
    {
      bail!(NOT_FOUND, "Cache not found or insufficient permissions");
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
  let permission = match access_type {
    AccessType::View => crate::permissions::CacheView::name(),
    AccessType::Edit => crate::permissions::CacheEdit::name(),
  };

  let hash_general_permission = GroupTable::new(db)
    .user_hash_permissions(user, permission)
    .await?;

  if !hash_general_permission {
    // If the user doesn't have the general permission, they must have specific access to the cache
    let group_ids = group_user::Entity::find()
      .filter(group_user::Column::UserId.eq(user))
      .select_only()
      .column_as(group_user::Column::GroupId, QueryAs::GroupId)
      .into_values::<Uuid, QueryAs>()
      .all(db)
      .await?;

    let mut user_cond = cache_access::Column::UserId.eq(user);
    let mut group_cond = cache_access::Column::GroupId.is_in(group_ids);
    let mut cond = Condition::any();

    // if only read access is requested, also include entries with edit access
    if access_type == AccessType::Edit {
      user_cond = user_cond.and(cache_access::Column::AccessType.eq(AccessType::Edit));
      group_cond = group_cond.and(cache_access::Column::AccessType.eq(AccessType::Edit));
    } else {
      // Only allow public caches if read access is sufficient. write requires explicit permissions
      cond = cond.add(cache::Column::Public.eq(true))
    }

    query = query.filter(cond.add(user_cond).add(group_cond));
  };

  Ok(query)
}
