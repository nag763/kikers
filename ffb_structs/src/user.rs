use crate::database::Database;
use crate::error::ApplicationError;

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct Model {
    pub id: u32,
    pub name: String,
    pub login: String,
    pub password: String,
    pub is_authorized: bool,
    pub role_id: u32,
}

pub struct Entity;

impl Entity {
    pub async fn get_favorite_leagues_id(id: u32) -> Result<Vec<u32>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let rows: Vec<(u32,)> = sqlx::query_as("SELECT league_id FROM USER_LEAGUE WHERE user_id=?")
            .bind(&id)
            .fetch_all(&mut conn)
            .await?;
        let result: Vec<u32> = rows.iter().map(|row| row.0).collect();
        Ok(result)
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

    pub async fn get_user_by_login(login: String) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model = sqlx::query_as("SELECT * FROM USER WHERE login=? LIMIT 1")
            .bind(&login)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
    }

    pub async fn get_user_by_credentials(
        login: String,
        password: String,
    ) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model =
            sqlx::query_as::<_, Model>("SELECT * FROM USER WHERE login=? AND password=? LIMIT 1")
                .bind(&login)
                .bind(&password)
                .fetch_optional(&mut conn)
                .await?;
        Ok(model)
    }

    pub async fn delete_user_id(id: u32) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("DELETE FROM USER WHERE id =?")
            .bind(&id)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn insert_user(
        login: String,
        name: String,
        password: String,
    ) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("INSERT INTO USER(login, name, password) VALUES(?,?,?)")
            .bind(&login)
            .bind(&name)
            .bind(&password)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn add_leagues_as_favorite(
        user_id: u32,
        league_id: u32,
    ) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("INSERT INTO USER_LEAGUE(user_id, league_id) VALUES(?,?)")
            .bind(&user_id)
            .bind(&league_id)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn remove_leagues_as_favorite(
        user_id: u32,
        league_id: u32,
    ) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("DELETE FROM USER_LEAGUE WHERE user_id=? AND league_id=?")
            .bind(&user_id)
            .bind(&league_id)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn change_activation_status(
        id: u32,
        is_authorized: bool,
    ) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("UPDATE USER SET is_authorized=? WHERE id =?")
            .bind(&is_authorized)
            .bind(&id)
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

    pub async fn login_exists(login: String) -> Result<bool, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let row: (bool,) =
            sqlx::query_as("SELECT IF(COUNT(id)!=0, TRUE, FALSE) FROM USER WHERE login=? LIMIT 1")
                .bind(&login)
                .fetch_one(&mut conn)
                .await?;
        Ok(row.0)
    }
}
