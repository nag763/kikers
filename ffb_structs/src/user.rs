use crate::database::Database;
use crate::error::ApplicationError;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Default, sqlx::FromRow)]
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

    pub async fn get_user_by_login(login: &str) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model = sqlx::query_as("SELECT * FROM USER WHERE login=? LIMIT 1")
            .bind(login)
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

    pub async fn delete_user_uuid(uuid: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("DELETE FROM USER WHERE uuid =?")
            .bind(&uuid)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn insert_user(
        login: &str,
        name: &str,
        password: &str,
    ) -> Result<(), ApplicationError> {
        let uuid = Uuid::new_v4();
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("INSERT INTO USER(uuid, login, name, password) VALUES(?, ?,?,?)")
            .bind(uuid.to_string())
            .bind(login)
            .bind(name)
            .bind(password)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn add_leagues_as_favorite(
        user_id: u32,
        league_id: u32,
    ) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let mut redis_conn = Database::acquire_redis_connection()?;
        sqlx::query("INSERT INTO USER_LEAGUE(user_id, league_id) VALUES(?,?)")
            .bind(&user_id)
            .bind(&league_id)
            .execute(&mut conn)
            .await?;
        redis::cmd("DEL")
            .arg(format!("fav_leagues:{}", user_id))
            .query(&mut redis_conn)?;
        Ok(())
    }

    pub async fn remove_leagues_as_favorite(
        user_id: u32,
        league_id: u32,
    ) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let mut redis_conn = Database::acquire_redis_connection()?;
        sqlx::query("DELETE FROM USER_LEAGUE WHERE user_id=? AND league_id=?")
            .bind(&user_id)
            .bind(&league_id)
            .execute(&mut conn)
            .await?;
        redis::cmd("DEL")
            .arg(format!("fav_leagues:{}", user_id))
            .query(&mut redis_conn)?;
        Ok(())
    }

    pub async fn change_activation_status(
        uuid: &str,
        is_authorized: bool,
    ) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("UPDATE USER SET is_authorized=? WHERE uuid =?")
            .bind(&is_authorized)
            .bind(uuid)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update(model: Model) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("UPDATE USER SET name=?,is_authorized=? WHERE id =?")
            .bind(&model.name)
            .bind(&model.is_authorized)
            .bind(&model.id)
            .execute(&mut conn)
            .await?;
        Ok(())
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
