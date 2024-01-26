pub mod session;

use axum::body::HttpBody;
use axum::extract::{FromRequestParts, Query};
use axum::headers::UserAgent;
use axum::http::{HeaderMap, Method, Request, Uri};
use axum::{async_trait, extract::FromRequest};
use axum::{BoxError, Form, TypedHeader};
use axum_extra::extract::CookieJar;
use hyper::header;
use serde::de::DeserializeOwned;
use std::str::FromStr;

use self::session::Session;
use super::{LOGIN_INDEX, SESSION_ID, SESSION_TOKEN_ID};
use crate::now_duration;
use crate::serve::error::ResponseError;

/// ChatGPT session Extension
pub struct SessionExt {
    pub session: Session,
    pub session_token: Option<String>,
    pub headers: HeaderMap,
    pub jar: CookieJar,
}

/// Arkose session Extension
pub struct ArkoseSessionExt<T: DeserializeOwned> {
    pub uri: Uri,
    pub method: Method,
    pub user_agent: TypedHeader<UserAgent>,
    pub query: Option<Query<T>>,
    pub headers: HeaderMap,
    pub session: Option<Session>,
    pub body: Option<Form<T>>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for SessionExt
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = ResponseError;

    async fn from_request(req: Request<B>, _: &S) -> Result<Self, Self::Rejection> {
        let (parts, _) = req.into_parts().into();

        // Extract session from cookie
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar
            .get(SESSION_ID)
            .ok_or(ResponseError::TempporaryRedirect(LOGIN_INDEX))?;
        let session = extract_session(cookie.value())?;

        // Compare the current timestamp with the expiration time of the session
        let current_timestamp = now_duration()
            .map_err(ResponseError::InternalServerError)?
            .as_secs() as i64;
        if current_timestamp > session.expires {
            return Err(ResponseError::TempporaryRedirect(LOGIN_INDEX));
        }

        // The access token may be expired
        Ok(SessionExt {
            session,
            session_token: jar.get(SESSION_TOKEN_ID).map(|c| c.value().to_owned()),
            jar,
            headers: parts.headers.clone(),
        })
    }
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for ArkoseSessionExt<T>
where
    T: DeserializeOwned + std::marker::Send,
    B: Send + 'static,
    S: Send + Sync,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ResponseError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        // Try to extract session from cookie
        let session = CookieJar::from_headers(&parts.headers)
            .get(SESSION_ID)
            .map(|v| extract_session(v.value()))
            .map(|v| v.ok())
            .flatten();

        let uri = parts.uri.clone();
        let method = parts.method.clone();
        let headers = parts.headers.clone();

        // Extract user agent
        let user_agent = TypedHeader::<UserAgent>::from_request_parts(&mut parts, state)
            .await
            .map_err(ResponseError::BadRequest)?;

        // Extract query
        let query = if uri.query().is_some() {
            Query::<T>::from_request_parts(&mut parts, state)
                .await
                .map_err(ResponseError::BadRequest)
                .map_or(None, |v| Some(v))
        } else {
            None
        };

        // Try to extract body if content type is form
        let body = if headers.get(header::CONTENT_TYPE).is_some() {
            Form::from_request(Request::from_parts(parts, body), state)
                .await
                .map_or(None, |v| Some(v))
        } else {
            None
        };

        Ok(ArkoseSessionExt {
            uri,
            method,
            query,
            user_agent,
            headers,
            session,
            body,
        })
    }
}

fn extract_session(cookie_value: &str) -> Result<Session, ResponseError> {
    Session::from_str(cookie_value)
        .map_err(|_| ResponseError::TempporaryRedirect(LOGIN_INDEX))
        .and_then(|session| {
            crate::token::check(&session.access_token)
                .map_err(|_| ResponseError::TempporaryRedirect(LOGIN_INDEX))
                .and_then(|_| Ok(session))
        })
}
