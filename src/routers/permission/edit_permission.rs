use std::str::FromStr;

use crate::{
    difference::{Difference, DifferenceContext, DifferenceResult},
    error::RouterError,
    models::{NewPermissionCondition, Permission, PermissionCondition},
    routers::permission::SimpleCondition,
    DbPool,
};
use actix_web::web::{self, Path};
use diesel::prelude::*;
use uuid::Uuid;

use super::NewPermissionData;

/// Edit's the target permission
///
/// On updating the conditions this router will
/// remove all related conditions, and insert the new ones
/// this solution is kind of stupid but it's really simple.
pub async fn edit_permission<'a>(
    target_permission: Path<String>,
    new_permission: web::Json<NewPermissionData>,
    pool: web::Data<DbPool>,
) -> Result<&'a str, RouterError> {
    use crate::schema::app_permission_conditions::dsl::{
        app_permission_conditions, id as condition_id, name as condition_name,
        value as condition_value,
    };
    use crate::schema::app_permissions::dsl::{
        action, app_permissions, object, subject, uuid as uuid_of_permission,
    };

    let target_permission = target_permission.into_inner();
    let new_permission = new_permission.into_inner();

    let result: Result<&'a str, RouterError> = web::block(move || {
        let mut conn = pool.get().unwrap();
        let permission_uuid = Uuid::from_str(&target_permission)?;

        let permission: Permission = diesel::update(app_permissions)
            .filter(uuid_of_permission.eq(permission_uuid))
            .set((
                subject.eq(new_permission.subject),
                object.eq(new_permission.object),
                action.eq(new_permission.action),
            ))
            .get_result(&mut conn)?;

        // Get existsing conditions
        let target_conditions: Vec<PermissionCondition> =
            PermissionCondition::belonging_to(&permission).get_results(&mut conn)?;

        // Turn PermissionCondition into SimpleCondition
        let target_conditions: Vec<SimpleCondition> = target_conditions
            .into_iter()
            .map(|condition| SimpleCondition::from(condition))
            .collect();

        // Provide required data
        let difference_context =
            DifferenceContext::new(target_conditions, new_permission.conditions);

        // Create Difference Object from context
        let mut difference = Difference::from(difference_context);

        // Found the difference between Existing conditions and new conditions,
        let difference_result = difference.diff();

        // Now we gonna walk the results and do what they say :)
        for diff_action in difference_result {
            match diff_action {
                DifferenceResult::Update(old, new) => {
                    diesel::update(app_permission_conditions.filter(condition_id.eq(old.id)))
                        .set((condition_name.eq(new.name), condition_value.eq(new.value)))
                        .execute(&mut conn)?;
                }
                DifferenceResult::Insert(new) => {
                    NewPermissionCondition {
                        name: new.name,
                        value: new.value,
                        permission_id: permission.id,
                    }
                    .insert_into(app_permission_conditions)
                    .execute(&mut conn)?;
                }
                DifferenceResult::Remove(old) => {
                    diesel::delete(app_permission_conditions.filter(condition_id.eq(old.id)))
                        .execute(&mut conn)?;
                }
            }
        }

        Ok("Updated")
    })
    .await
    .unwrap();

    result
}
