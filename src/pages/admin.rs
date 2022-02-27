use crate::auth::JwtUser;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

use crate::entities::{user, user::Model as User};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect};

#[derive(Template, Debug)]
#[template(path = "admin.html")]
struct Admin {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    data: Vec<User>,
    offset: u64,
}

#[get("/admin")]
pub async fn admin_dashboard(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let db_url = std::env::var("DATABASE_URL")?;
    let conn = sea_orm::Database::connect(&db_url).await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let offset: u64 = context_query.offset.unwrap_or_else(|| 0);
    let data: Vec<User> = user::Entity::find()
        .filter(Condition::all().add(user::Column::Role.lt(jwt_user.role)))
        .limit(10)
        .offset(context_query.offset.unwrap_or_else(|| 0))
        .all(&conn)
        .await?;
    let index = Admin {
        title: "User management".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        data,
        offset,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
