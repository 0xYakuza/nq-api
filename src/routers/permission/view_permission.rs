use std::str::FromStr;

use crate::{
    error::RouterError,
    models::{Permission, PermissionCondition},
    routers::{
        multip,
        permission::{PermissionWithConditions, SimplePermission},
    },
    DbPool,
};
use actix_web::web;
use diesel::prelude::*;
use uuid::Uuid;

/// Returns the list of Permissions
///
/// with related Conditions
pub async fn get_permission(
    path: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<web::Json<PermissionWithConditions>, RouterError> {
    use crate::schema::app_permissions::dsl::{app_permissions, uuid as uuid_from_permissions};

    let path = path.into_inner();

    let permission: Result<PermissionWithConditions, RouterError> = web::block(move || {
        let mut conn = pool.get().unwrap();
        let uuid = Uuid::from_str(&path)?;

        let permission_by_uuid: Permission = app_permissions
            .filter(uuid_from_permissions.eq(uuid))
            .first(&mut conn)?;

        let conditions: Vec<PermissionCondition> =
            PermissionCondition::belonging_to(&permission_by_uuid).load(&mut conn)?;

        Ok(PermissionWithConditions {
            permission: SimplePermission::from(permission_by_uuid),
            conditions,
        })
    })
    .await
    .unwrap();

    Ok(web::Json(permission?))
}
