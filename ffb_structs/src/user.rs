//! The user is a MySQL stored client of the application.
//!
//! A user can be identified either by his id, his uuid, and his login.
//! He has a role, that defines the set of actions he can execute within the 
//! application and a locale, that defines the language that will be used to 
//! translate the application when used.
//!
//! His password should always be passed as encrypted to the methods requesting 
//! asking for it as input.

use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use uuid::Uuid;

#[derive(
    Clone, Debug, PartialEq, Default, Display, sqlx::FromRow, serde::Serialize, serde::Deserialize,
)]
#[display(fmt = "#{} named {} with login {}", id, name, login)]
pub struct Model {
    /// His in base id.
    pub id: u32,
    /// His uuid, usually used for update and modifications request in order
    /// to mitigate the possible right elevation.
    pub uuid: String,
    /// The user name, up to the user to pick whatever he wants.
    pub name: String,
    /// The user's login, used to identify himself.
    pub login: String,
    /// His password, should always be encrypted.
    pub password: String,
    /// Whether th euser is authorized or not.
    ///
    /// An unauthorized user can't use the application nor log in.
    pub is_authorized: bool,
    /// His locale, used to translate the GUIs.
    pub locale_id: u32,
    /// His role, used to determine what are his rights within the application.
    pub role_id: u32,
}

pub struct Entity;

impl Entity {
    
    /// Get the favorite leagues id of a user's id.
    ///
    /// This method is cached onto Redis.
    ///
    /// # Arguments
    ///
    /// - id : The user's in base id.
    pub async fn get_favorite_leagues_id(id: u32) -> Result<Vec<u32>, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let fav_leagues_as_string: Option<String> = redis::cmd("GET")
            .arg(format!("fav_leagues:{}", id))
            .query(&mut redis_conn)?;
        let fav_leagues = match fav_leagues_as_string {
            Some(v) => {
                debug!("The favorite leagues have been found in cache");
                let fav_leagues: Vec<u32> = serde_json::from_str(v.as_str())?;
                fav_leagues
            }
            None => {
                let mut conn = Database::acquire_sql_connection().await?;
                debug!("The favorite leagues haven't been found in cache, requesting them from the database.");
                let rows: Vec<(u32,)> =
                    sqlx::query_as("SELECT league_id FROM USER_LEAGUE WHERE user_id=?")
                        .bind(&id)
                        .fetch_all(&mut conn)
                        .await?;
                let result: Vec<u32> = rows.iter().map(|row| row.0).collect();
                redis::cmd("SET")
                    .arg(format!("fav_leagues:{}", id))
                    .arg(&serde_json::to_string(&result)?)
                    .arg("EX")
                    .arg(3600)
                    .query(&mut redis_conn)?;
                debug!("The favorite leagues id have been successfully fetched and cached within the database");
                result
            }
        };
        Ok(fav_leagues)
    }

    pub async fn get_favorite_clubs_id(id: u32) -> Result<Vec<u32>, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let fav_leagues_as_string: Option<String> = redis::cmd("GET")
            .arg(format!("fav_clubs:{}", id))
            .query(&mut redis_conn)?;
        let fav_clubs = match fav_leagues_as_string {
            Some(v) => {
                let fav_clubs: Vec<u32> = serde_json::from_str(v.as_str())?;
                debug!("The favorite clubs have been found in cache");
                fav_clubs
            }
            None => {
                let mut conn = Database::acquire_sql_connection().await?;
                debug!("The favorite clubs haven't been found in cache");
                let rows: Vec<(u32,)> =
                    sqlx::query_as("SELECT club_id FROM USER_CLUB WHERE user_id=?")
                        .bind(&id)
                        .fetch_all(&mut conn)
                        .await?;
                let result: Vec<u32> = rows.iter().map(|row| row.0).collect();
                redis::cmd("SET")
                    .arg(format!("fav_clubs:{}", id))
                    .arg(&serde_json::to_string(&result)?)
                    .arg("EX")
                    .arg(3600)
                    .query(&mut redis_conn)?;
                debug!("The favorite clubs have been successfully fetched from the database and stored in the cache");
                result
            }
        };
        Ok(fav_clubs)
    }

    /// Get the users paginated.
    ///
    /// This method allows to get a list of user who have a lower role than the
    /// requester's. The results are paginated through the per page and page 
    /// arguments.
    ///
    /// # Arguments
    ///
    /// - role : the user's role, a user shouldn't be able to see the users with
    /// a lower role.
    /// - per_page : the number of results returned per pages.
    /// - page : the page requested, needs to remain coherent with per_page 
    /// variable.
    pub async fn get_users_with_pagination(
        role: u32,
        per_page: u32,
        page: u32,
    ) -> Result<Vec<Model>, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let redis_key: String = format!("users:{}::{}::{}", role, per_page, page);
        let paginated_users_as_string: Option<String> = redis::cmd("GETEX")
            .arg(&redis_key)
            .arg("EX")
            .arg(250)
            .query(&mut redis_conn)?;
        if let Some(paginated_users_as_string) = paginated_users_as_string {
            let models: Vec<Model> = serde_json::from_str(paginated_users_as_string.as_str())?;
            Ok(models)
        } else {
            let mut conn = Database::acquire_sql_connection().await?;
            let offset = per_page * page;
            let models = sqlx::query_as::<_, Model>(
                "SELECT * FROM USER WHERE role_id < ? ORDER BY id LIMIT ?,?",
            )
            .bind(&role)
            .bind(&offset)
            .bind(&per_page)
            .fetch_all(&mut conn)
            .await?;
            redis::cmd("SET")
                .arg(&redis_key)
                .arg(&serde_json::to_string(&models)?)
                .arg("EX")
                .arg(200)
                .query(&mut redis_conn)?;
            Ok(models)
        }
    }

    /// Find a user by id.
    ///
    /// The role check is needed in order to ensure that there isn't any right 
    /// escalation.
    ///
    /// # Arguments
    /// - id : the user requested.
    /// - role_id : the role of the requester.
    pub async fn find_by_id_with_role_check(
        id: u32,
        role_id: u32,
    ) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model =
            sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE id=? AND role_id < ? LIMIT 1")
                .bind(&id)
                .bind(&role_id)
                .fetch_optional(&mut conn)
                .await?;
        Ok(model)
    }

    /// Find a user by his id.
    ///
    /// # Arguments
    ///
    /// - id : The user's in base id we are looking for.
    pub async fn find_by_id(id: u32) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model = sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE id=? LIMIT 1")
            .bind(&id)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
    }

    /// Find a user by his UUID.
    ///
    /// Can be useful in case we need to add a layer of security for more 
    /// destructive requests.
    ///
    /// # Arguments
    ///
    /// - uuid : the user's uuid.
    pub async fn find_by_uuid(uuid: &str) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model = sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE uuid=? LIMIT 1")
            .bind(uuid)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
    }

    /// Get a user by login.
    ///
    /// The role check is used to restrict the access for more destructive
    /// subrequests.
    ///
    /// # Arguments
    ///
    /// - login : The requested user's login.
    /// - role_id : The requester's role id.
    pub async fn get_user_by_login_with_role_check(
        login: &str,
        role_id: u32,
    ) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model = sqlx::query_as("SELECT * FROM USER WHERE login=? AND role_id < ? LIMIT 1")
            .bind(login)
            .bind(role_id)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
    }

    /// Get a user by his credentials.
    ///
    /// This method is mainly used to authentify a user.
    ///
    /// The password needs to be hashed.
    ///
    /// # Arguments
    ///
    /// - login : The requested user's login.
    /// - password : The requested user's encrypted password.
    pub async fn get_user_by_credentials(
        login: &str,
        password: &str,
    ) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model =
            sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE login=? AND password=? LIMIT 1")
                .bind(login)
                .bind(password)
                .fetch_optional(&mut conn)
                .await?;
        Ok(model)
    }

    /// Delete a user with a role check prior the update.
    ///
    /// If the role of the requester isn't higher than the deleted user, nothing
    /// will be deleted.
    ///
    /// # Arguments
    ///
    /// - user_uuid : The requested deleted user's inbase uuid.
    /// - role_id : The requester's role id.
    pub async fn delete_user_uuid_with_role_check(
        user_uuid: &str,
        role_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let mut redis_conn = Database::acquire_redis_connection()?;
        let result = sqlx::query("DELETE FROM USER WHERE uuid =? AND role_id < ?")
            .bind(&user_uuid)
            .bind(&role_id)
            .execute(&mut conn)
            .await?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg("users:*").query(&mut redis_conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut redis_conn)?;
        }
        info!("User {} has been deleted", user_uuid);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Create a new user in database.
    ///
    /// # Arguments
    /// - login : the user's login.
    /// - name : his name within the application.
    /// - locale_id : the locale he will use while browsing the site.
    /// - password : his password, encrypted.
    pub async fn insert_user(
        login: &str,
        name: &str,
        locale_id: u32,
        password: &str,
    ) -> Result<TransactionResult, ApplicationError> {
        let gen_uuid = Uuid::new_v4();
        let mut conn = Database::acquire_sql_connection().await?;
        let mut redis_conn = Database::acquire_redis_connection()?;
        let result = sqlx::query(
            "INSERT INTO USER(uuid, login, name, locale_id, password) VALUES(?, ?,?,?,?)",
        )
        .bind(gen_uuid.to_string())
        .bind(login)
        .bind(name)
        .bind(locale_id)
        .bind(password)
        .execute(&mut conn)
        .await?;
        info!("User {} has been created", login);
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg("users:*").query(&mut redis_conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut redis_conn)?;
        }
        info!("User {} has been inserted", gen_uuid);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Add leagues as favorite for a given user.
    ///
    /// The leagues ID are stored directly within the MongoDB structs.
    ///
    /// # Arguments
    ///
    /// - user_id : the user to add a league as favorite for.
    /// - league_id : the league id to add as a favorite for the user.
    pub async fn add_leagues_as_favorite(
        user_id: u32,
        league_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let mut redis_conn = Database::acquire_redis_connection()?;
        let result = sqlx::query("INSERT INTO USER_LEAGUE(user_id, league_id) VALUES(?,?)")
            .bind(&user_id)
            .bind(&league_id)
            .execute(&mut conn)
            .await?;
        redis::cmd("DEL")
            .arg(format!("fav_leagues:{}", user_id))
            .query(&mut redis_conn)?;
        debug!("The league {} has been added to the favorites of user {}", league_id, user_id);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Remove a league to the user's favorite.
    ///
    /// # Arguments
    ///
    /// - user_id : The user to remove a favorite league for.
    /// - league_id : The league id to remove as favorite for the user.
    pub async fn remove_leagues_as_favorite(
        user_id: u32,
        league_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let mut redis_conn = Database::acquire_redis_connection()?;
        let result = sqlx::query("DELETE FROM USER_LEAGUE WHERE user_id=? AND league_id=?")
            .bind(&user_id)
            .bind(&league_id)
            .execute(&mut conn)
            .await?;
        redis::cmd("DEL")
            .arg(format!("fav_leagues:{}", user_id))
            .query(&mut redis_conn)?;
        debug!("The league {} has been removed to the favorites of user {}", league_id, user_id);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Add a club as favorite for the given user.
    ///
    /// # Arguments
    /// 
    /// - user_id : the id of the user to add a club as favorites.
    /// - club_id : The club to add as favorite for the user.
    pub async fn add_club_as_favorite(
        user_id: u32,
        club_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("INSERT INTO USER_CLUB(user_id, club_id) VALUES(?,?)")
            .bind(&user_id)
            .bind(&club_id)
            .execute(&mut conn)
            .await?;
        redis::cmd("DEL")
            .arg(format!("fav_clubs:{}", user_id))
            .query(&mut redis_conn)?;
        debug!("The club {} has been added to the favorites of user {}", club_id, user_id);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Remove a club from the user's favorites.
    ///
    /// # Arguments
    ///
    /// - user_id : The user who needs to get a club removed from his favorites.
    /// - club_id : The club that needs to get removed from the profile.
    pub async fn remove_club_as_favorite(
        user_id: u32,
        club_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("DELETE FROM USER_CLUB WHERE user_id=? AND club_id=? LIMIT 1")
            .bind(&user_id)
            .bind(&club_id)
            .execute(&mut conn)
            .await?;
        redis::cmd("DEL")
            .arg(format!("fav_clubs:{}", user_id))
            .query(&mut redis_conn)?;
        debug!("The club {} has been removed from the favorites of user {}", club_id, user_id);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Change the activation status for the given user with role check.
    ///
    /// # Arguments
    ///
    /// - uuid : The uuid of the requested user that needs to be activated.
    /// - is_authorized : The requested user's new authorization status.
    /// - role_id : The role id of the requester.
    pub async fn change_activation_status_with_role_check(
        uuid: &str,
        is_authorized: bool,
        role_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let mut redis_conn = Database::acquire_redis_connection()?;
        let result = sqlx::query("UPDATE USER SET is_authorized=? WHERE uuid =? AND role_id < ?")
            .bind(&is_authorized)
            .bind(uuid)
            .bind(role_id)
            .execute(&mut conn)
            .await?;
        info!(
            "User#{} activation status have been updated to {}",
            uuid, is_authorized
        );
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg("users:*").query(&mut redis_conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut redis_conn)?;
        }
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Update a user with role check.
    ///
    /// # Arguments
    ///
    /// - model : The requested model to update.
    /// - role_id : The requester's role id.
    pub async fn update_with_role_check(
        model: Model,
        role_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query(
            "UPDATE USER SET name=?,is_authorized=?,role_id=? WHERE id =? and role_id < ?",
        )
        .bind(&model.name)
        .bind(&model.is_authorized)
        .bind(&model.role_id)
        .bind(&model.id)
        .bind(&role_id)
        .execute(&mut conn)
        .await?;
        info!("User {} has been updated", &model.login);
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg("users:*").query(&mut redis_conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut redis_conn)?;
        }
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Updates a self user.
    ///
    /// This method is used for a user to update his profile.
    ///
    /// This method should **never** bind the variables `role_id` nor 
    /// `is_authorized`.
    ///
    /// # Arguments
    ///
    /// 
    pub async fn update_self(model: Model) -> Result<TransactionResult, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("UPDATE USER SET name=?,password=?, locale_id=? WHERE id =?")
            .bind(&model.name)
            .bind(&model.password)
            .bind(model.locale_id)
            .bind(&model.id)
            .execute(&mut conn)
            .await?;
        info!("User {} has updated himself", &model.login);
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg("users:*").query(&mut redis_conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut redis_conn)?;
        }
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Lookup whether the given login exists in database.
    ///
    /// # Arguments
    ///
    /// - login : The login to lookup for.
    pub async fn login_exists(login: &str) -> Result<bool, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let row: (bool,) =
            sqlx::query_as("SELECT IF(COUNT(id)!=0, TRUE, FALSE) FROM USER WHERE login=? LIMIT 1")
                .bind(login)
                .fetch_one(&mut conn)
                .await?;
        Ok(row.0)
    }
}

