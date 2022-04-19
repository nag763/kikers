use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Default, Display, sqlx::FromRow)]
#[display(fmt = "#{} named {} with login {}", id, name, login)]
pub struct Model {
    pub id: u32,
    pub uuid: String,
    pub name: String,
    pub login: String,
    pub password: String,
    pub is_authorized: bool,
    pub role_id: u32,
}

pub struct Entity;

impl Entity {
    pub async fn get_favorite_leagues_id(id: u32) -> Result<Vec<u32>, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let fav_leagues_as_string: Option<String> = redis::cmd("GET")
            .arg(format!("fav_leagues:{}", id))
            .query(&mut redis_conn)?;
        let fav_leagues = match fav_leagues_as_string {
            Some(v) => {
                let fav_leagues: Vec<u32> = serde_json::from_str(v.as_str())?;
                fav_leagues
            }
            None => {
                let mut conn = Database::acquire_sql_connection().await?;
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
                result
            }
        };
        Ok(fav_leagues)
    }

    pub async fn get_users_with_pagination(
        role: u32,
        per_page: u32,
        page: u32,
    ) -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let offset = per_page * page;
        let models = sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE role_id < ? LIMIT ?,?")
            .bind(&role)
            .bind(&offset)
            .bind(&per_page)
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }

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

    pub async fn find_by_id(id: u32) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model = sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE id=? LIMIT 1")
            .bind(&id)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
    }

    pub async fn find_by_uuid(uuid: &str) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model = sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE uuid=? LIMIT 1")
            .bind(uuid)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
    }

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

    pub async fn delete_user_uuid_with_role_check(
        uuid: &str,
        role_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("DELETE FROM USER WHERE uuid =? AND role_id < ?")
            .bind(&uuid)
            .bind(&role_id)
            .execute(&mut conn)
            .await?;
        info!("User {} has been deleted", uuid);
        Ok(TransactionResult::from_expected_affected_rows(result, 1))
    }

    pub async fn insert_user(
        login: &str,
        name: &str,
        password: &str,
    ) -> Result<TransactionResult, ApplicationError> {
        let uuid = Uuid::new_v4();
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("INSERT INTO USER(uuid, login, name, password) VALUES(?, ?,?,?)")
            .bind(uuid.to_string())
            .bind(login)
            .bind(name)
            .bind(password)
            .execute(&mut conn)
            .await?;
        info!("User {} has been created", login);
        Ok(TransactionResult::from_expected_affected_rows(result, 1))
    }

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
        Ok(TransactionResult::from_expected_affected_rows(result, 1))
    }

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
        Ok(TransactionResult::from_expected_affected_rows(result, 1))
    }

    pub async fn change_activation_status_with_role_check(
        uuid: &str,
        is_authorized: bool,
        role_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
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
        Ok(TransactionResult::from_expected_affected_rows(result, 1))
    }

    pub async fn update_with_role_check(
        model: Model,
        role_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let result =
            sqlx::query("UPDATE USER SET name=?,is_authorized=? WHERE id =? and role_id < ?")
                .bind(&model.name)
                .bind(&model.is_authorized)
                .bind(&model.id)
                .bind(&role_id)
                .execute(&mut conn)
                .await?;
        info!("User {} has been updated", &model.login);
        Ok(TransactionResult::from_expected_affected_rows(result, 1))
    }

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
