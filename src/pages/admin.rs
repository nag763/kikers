use crate::auth::JwtUser;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

use crate::entities::{user, user::Model as User};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use sea_orm::PaginatorTrait;

#[derive(Template, Debug)]
#[template(path = "admin.html")]
struct Admin {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    data: Vec<User>,
    page: usize,
    per_page: usize,
    num_pages: usize,
}

#[get("/admin")]
pub async fn admin_dashboard(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let db_url = std::env::var("DATABASE_URL")?;
    let conn = sea_orm::Database::connect(&db_url).await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let page: usize = context_query.page.unwrap_or(0);
    let per_page: usize = context_query.per_page.unwrap_or(10);
    let paginated_data = user::Entity::find()
        .filter(Condition::all().add(user::Column::Role.lt(jwt_user.role)))
        .paginate(&conn, per_page);
    let num_pages = paginated_data.num_pages().await?;
    let data : Vec<User> =  paginated_data.fetch_page(page).await?;
    let index = Admin {
        title: "User management".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        data,
        page,
        num_pages,
        per_page
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
