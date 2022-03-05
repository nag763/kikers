use crate::auth::JwtUser;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

use sea_orm::{Statement, FromQueryResult};

#[derive(Template, Debug)]
#[template(path = "games/games.html")]
struct Games {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    three_next_games: Vec<GameCustom>,
}

#[derive(Debug, FromQueryResult)]
struct GameCustom {
    id: i32,
    home_team_odds: f32,
    away_team_odds: f32,
    home_team_name: String,
    away_team_name: String,
    stadium_name: String,
    stadium_city: String,
    stadium_country: String,
    competition_name: String,
    played_on: chrono::DateTime<chrono::FixedOffset>,
}

#[get("/games")]
pub async fn games(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let db_url = std::env::var("DATABASE_URL")?;
    let conn = sea_orm::Database::connect(&db_url).await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let now = time::OffsetDateTime::now_utc();
    // Join aliases not implemented yet for sea_orm:6.0
    let three_next_games: Vec<GameCustom> =
        GameCustom::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"
        SELECT
  `GAME`.`id`,
  `GAME`.`home_team_odds`,
  `GAME`.`away_team_odds`,
   home.`name` AS `home_team_name`,
   away.`name` AS `away_team_name`,
  `STADIUM`.`name` AS `stadium_name`,
  `STADIUM`.`city` AS `stadium_city`,
  `STADIUM`.`country` AS `stadium_country`,
  `COMPETITION`.`name` AS `competition_name`,
  `GAME`.`played_on`
FROM
  `GAME`
  INNER JOIN `CLUB` home ON `GAME`.`home_team_id` = home.`id`
  INNER JOIN `CLUB` away ON `GAME`.`away_team_id` = away.`id`
  INNER JOIN `EDITION` ON `GAME`.`edition_id` = `EDITION`.`id`
  INNER JOIN `STADIUM` ON `GAME`.`stadium_id` = `STADIUM`.`id`
  INNER JOIN `COMPETITION` ON `EDITION`.`competition_id` = `COMPETITION`.`id`
WHERE
  `GAME`.`home_team_score` IS NULL
  AND `GAME`.`home_team_odds` IS NOT NULL
  AND `GAME`.`away_team_score` IS NULL
  AND `GAME`.`away_team_odds` IS NOT NULL
  AND `GAME`.`edition_id` IS NOT NULL
  AND `GAME`.`played_on` IS NOT NULL
  AND `GAME`.`played_on` >= ?
  AND `GAME`.`result` IS NULL
ORDER BY
  `GAME`.`played_on` ASC
  LIMIT 3;
        "#,
            vec![now.to_string().into()],
        ))
        .into_model::<GameCustom>()
        .all(&conn)
        .await?;
    let index = Games {
        title: "Games".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        three_next_games,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
