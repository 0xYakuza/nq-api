use crate::error::RouterError;
use crate::models::{Organization, User};
use crate::select_model::SelectModel;
use crate::DbPool;
use actix_web::web;
use async_trait::async_trait;
use auth_z::{CheckPermission, GetModel, ModelPermission, ParsedPath};
use diesel::prelude::*;

const ANY_FILTER: char = '*';

#[derive(Debug)]
/// Request Action
enum Action {
    /// Create (POST) request to a controller
    Create,

    /// Edit (POST) request with id to a controller
    Edit,

    /// Delete (DELETE) request with id to a controller
    Delete,

    /// View (GET) request to a controller, id is not required
    View,
}

impl Action {
    fn from_auth_z<'a>(path: &ParsedPath, method: &'a str) -> Self {
        // Checks the id of path and request method
        match (path.id.clone(), method) {
            (Some(_), "GET") => Self::View,
            (None, "POST") => Self::Create,
            (Some(_), "POST") => Self::Edit,
            (Some(_), "DELETE") => Self::Delete,
            (None, _) => Self::View,
            (Some(_), _) => Self::View,
        }
    }
}

impl Into<&str> for Action {
    fn into(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Edit => "edit",
            Self::Delete => "delete",
            Self::View => "view",
        }
    }
}

#[derive(Debug, Clone)]
/// Actual Context of AuthZ
pub struct AuthZController {
    db_pool: DbPool,
}

impl AuthZController {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl CheckPermission for AuthZController {
    async fn check(&self, subject: Option<u32>, path: ParsedPath, method: String) -> bool {
        use crate::schema::app_permission_conditions::dsl::{
            app_permission_conditions, name, value,
        };
        use crate::schema::app_permissions::dsl::{
            action as permission_action, app_permissions, id as permission_id,
            object as permission_object, subject as permission_subject,
        };

        // these will be moved to the web::block closure
        let subject_copy = subject.clone();
        let path_copy = path.clone();

        let mut conn = self.db_pool.get().unwrap();
        let select_result: Result<(Vec<i32>, Vec<(String, String)>), RouterError> =
            web::block(move || {
                // Default subject query
                let subject_query = match subject_copy {
                    Some(subject) => vec![subject.to_string(), ANY_FILTER.to_string()],
                    None => vec![ANY_FILTER.to_string()],
                };

                // Foundout the requested Action
                let calculated_action = Action::from_auth_z(&path_copy, method.as_str());

                // Check the permissions and get the conditions
                let permissions_filter = app_permissions
                    .filter(permission_subject.eq_any(subject_query))
                    .filter(permission_object.eq(path_copy.controller.unwrap().clone()))
                    .filter(permission_action.eq::<&str>(calculated_action.into()));

                let permissions = permissions_filter
                    .clone()
                    .select(permission_id)
                    .load(&mut conn)?;

                let conditions = permissions_filter
                    .inner_join(app_permission_conditions)
                    .select((name, value))
                    .load(&mut conn)?;

                Ok((permissions, conditions))
            })
            .await
            .unwrap();

        let Ok(select_result) = select_result else {
            return false;
        };

        if select_result.0.is_empty() {
            return false;
        }

        // No need to Checking the conditions
        // there is no condition
        if select_result.1.is_empty() {
            return true;
        }

        // *Now Check the conditions*

        // First get the required Resource as Model
        let model = self
            .get_model(
                &path.controller.unwrap().clone(),
                path.id.unwrap().clone().parse().unwrap(),
            )
            .await;

        // We Got the model now we check every condition
        for (cond_name, cond_value) in select_result.1 {
            let Ok(model_attr) = ModelAttrib::try_from(cond_name.as_str()) else {
                return false
            };

            let attr = model.get_attr(model_attr.clone()).await;

            let inner_subject = match subject {
                Some(id) => Some(id.to_string()),
                None => None,
            };

            let result = ModelAttribResult::from(model_attr).validate(
                attr,
                inner_subject.as_deref(),
                &cond_value,
            );

            if result {
                return true;
            }
        }

        false
    }
}

#[async_trait]
impl GetModel<ModelAttrib, i32> for AuthZController {
    async fn get_model(
        &self,
        resource_name: &str,
        resource_id: u32,
    ) -> Box<dyn ModelPermission<ModelAttrib, i32>> {
        //let mut conn = self.db_pool.get().unwrap();
        let resource_id = resource_id as i32;

        // Resource must have been impl the Model permission trait
        let model: Box<dyn ModelPermission<ModelAttrib, i32>> = match resource_name {
            "user" => Box::new(User::from_id(self.db_pool.clone(), resource_id).await),

            "organization" => {
                Box::new(Organization::from_id(self.db_pool.clone(), resource_id).await)
            }

            _ => todo!(),
        };

        model
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionValueType {
    Boolean,
}

impl TryFrom<&str> for ConditionValueType {
    type Error = RouterError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "true" | "false" => Ok(Self::Boolean),

            _ => Err(RouterError::BadRequest(
                "Condition Value Type is not correct!".to_string(),
            )),
        }
    }
}

pub trait Condition<'a> {
    /// Validates the condition based on subject and value
    fn validate(
        &self,
        attribute: Option<i32>,
        subject: Option<&'a str>,
        condition_value: &'a str,
    ) -> bool
    where
        Self: Sized;

    /// Returns the value type of the condition
    fn get_value_type(&self) -> ConditionValueType
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct Owner;

impl<'a> Condition<'a> for Owner {
    // Validates the Owner Condition
    fn validate(
        &self,
        attr: Option<i32>,
        subject: Option<&'a str>,
        condition_value: &'a str,
    ) -> bool {
        let Some(subject) = subject else {
            return false;
        };

        if condition_value == "true" {
            matches!(attr, Some(_)) && subject == attr.unwrap().to_string()
        } else if condition_value == "false" {
            matches!(attr, None) || subject != attr.unwrap().to_string()
        } else {
            true
        }
    }

    fn get_value_type(&self) -> ConditionValueType {
        ConditionValueType::Boolean
    }
}

#[derive(Debug, Clone)]
pub enum ModelAttribResult {
    /// Owner Condition Result
    Owner(Owner),
}

impl<'a> Condition<'a> for ModelAttribResult {
    fn validate(
        &self,
        attribute: Option<i32>,
        subject: Option<&'a str>,
        condition_value: &'a str,
    ) -> bool {
        match self {
            Self::Owner(owner) => owner.validate(attribute, subject, condition_value),
        }
    }

    fn get_value_type(&self) -> ConditionValueType {
        match self {
            Self::Owner(owner) => owner.get_value_type(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModelAttrib {
    Owner,
}

impl From<ModelAttrib> for ModelAttribResult {
    // From ModelAttrib return the Result Enum, so we can
    // validate the Condition
    fn from(value: ModelAttrib) -> Self {
        match value {
            ModelAttrib::Owner => ModelAttribResult::Owner(Owner {}),
        }
    }
}

// Maybe we can use TryFrom
impl TryFrom<&str> for ModelAttrib {
    type Error = RouterError;
    // Returns ModelAttrib from &str (string)
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "isOwner" => Ok(Self::Owner),

            v => Err(RouterError::BadRequest(format!(
                "Condition with name {} not found!",
                v
            ))),
        }
    }
}

#[async_trait]
impl ModelPermission<ModelAttrib, i32> for User {
    async fn get_attr(&self, name: ModelAttrib) -> Option<i32> {
        match name {
            ModelAttrib::Owner => Some(self.account_id),
        }
    }
}

#[async_trait]
impl ModelPermission<ModelAttrib, i32> for Organization {
    async fn get_attr(&self, name: ModelAttrib) -> Option<i32> {
        match name {
            ModelAttrib::Owner => Some(self.owner_account_id),
        }
    }
}
