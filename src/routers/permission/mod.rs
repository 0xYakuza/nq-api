use crate::{
    authz::{Condition, ConditionValueType, ModelAttrib, ModelAttribResult},
    error::RouterError,
    models::{Permission, PermissionCondition},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod add_permission;
pub mod edit_permission;
pub mod permissions_list;
pub mod view_permission;
pub mod delete_permission;

#[derive(Serialize, Deserialize)]
pub struct NewPermissionData {
    subject: String,
    object: String,
    action: String,
    conditions: Vec<SimpleCondition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleCondition {
    name: String,
    value: String,
}

impl SimpleCondition {
    fn validate(&self) -> Result<(), RouterError> {
        let model_attr = ModelAttrib::try_from(self.name.as_str())?;
        let attr_result = ModelAttribResult::from(model_attr);
        let value_type = attr_result.get_value_type();

        let self_value_type = ConditionValueType::try_from(self.value.as_str())?;

        if value_type != self_value_type {
            return Err(RouterError::BadRequest(
                "Condition value type is not correct!".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Serialize, Eq, Ord, Hash, Debug, Clone, PartialEq, PartialOrd)]
pub struct SimplePermission {
    uuid: Uuid,
    subject: String,
    object: String,
    action: String,
}

impl From<Permission> for SimplePermission {
    fn from(value: Permission) -> Self {
        Self {
            uuid: value.uuid,
            subject: value.subject,
            object: value.object,
            action: value.action,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PermissionWithConditions {
    #[serde(flatten)]
    permission: SimplePermission,
    conditions: Vec<PermissionCondition>,
}
