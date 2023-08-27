use crate::models::User;
use crate::user::FullUserProfile;
use crate::{error::RouterError, DbPool};
use actix_web::web;
use diesel::prelude::*;
use uuid::Uuid;

/// Returns the list of all users
pub async fn users_list(
    pool: web::Data<DbPool>,
) -> Result<web::Json<Vec<FullUserProfile>>, RouterError> {
    use crate::schema::app_accounts::dsl::{
        app_accounts, username as account_username, uuid as uuid_of_account,
    };
    use crate::schema::app_emails::dsl::{app_emails, email as email_address};
    use crate::schema::app_user_names::dsl::{
        app_user_names, first_name as f_name, last_name as l_name, primary_name,
    };
    use crate::schema::app_users::dsl::app_users;

    let users_list: Result<Vec<FullUserProfile>, RouterError> = web::block(move || {
        let mut conn = pool.get().unwrap();

        // What is this :|
        // I know this is ugly but 
        // this is the best way to make query in this situation
        //
        // good luck if you gonna read this :)
        let users: Vec<(Uuid, String, User, String, String, String)> = app_users
            .inner_join(
                // Join the accounts, emails and user_names
                // tables together
                app_accounts
                    .inner_join(app_emails)
                    .inner_join(app_user_names),
            )
            // We only want the primary user name
            .filter(primary_name.eq(true))
            .select((
                // select the uuid of account
                uuid_of_account,
                // username of the account
                account_username,
                // The User Model
                User::as_select(),
                // User's primary email
                email_address,
                // First name from names table
                f_name,
                // and last name
                l_name,
            ))
            .load(&mut conn)?;

        let users: Vec<FullUserProfile> = users
            .into_iter()
            .map(
                |(uuid, username, user, email, first_name, last_name)| FullUserProfile {
                    uuid: uuid.to_string(),
                    email,
                    username,
                    birthday: user.birthday,
                    last_name: Some(last_name),
                    first_name: Some(first_name),
                    profile_image: user.profile_image,
                },
            )
            .collect();

        Ok(users)
    })
    .await
    .unwrap();

    Ok(web::Json(users_list?))
}
