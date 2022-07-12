use crate::accounts::Account;
use crate::constants::Value;
use crate::request;
use jelly::actix_session::UserSession;
use jelly::actix_web::http::header;
use jelly::actix_web::http::header::ContentType;
use jelly::prelude::*;
use jelly::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoggedInUser {
    pub id: i32,
    pub name: String,
    pub email: String,
}

pub async fn get_logged_in_user(request: HttpRequest) -> Result<HttpResponse> {
    if !request::is_authenticated(&request).await? {
        request.get_session().clear();
        return Ok(HttpResponse::Ok()
            .set(ContentType::json())
            .header(header::SET_COOKIE, Value::RememberMeTokenInvalidate)
            .body("{}"));
    }
    let user = request.user()?;
    let db = request.db_pool()?;
    let account = Account::get(user.id, db).await;
    if let Ok(account) = account {
        Ok(HttpResponse::Ok()
            .set(ContentType::json())
            .json(&LoggedInUser {
                id: account.id,
                name: account.name,
                email: account.email,
            }))
    } else {
        request.get_session().clear();
        Ok(HttpResponse::Ok()
            .set(ContentType::json())
            .header(header::SET_COOKIE, Value::RememberMeTokenInvalidate)
            .body("{}"))
    }
}
