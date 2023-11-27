use std::convert::TryFrom;
use std::sync::Arc;
use base64::Engine;

use once_cell::sync::Lazy;
use regex::Regex;
use warp::{Filter, reject, Rejection};

#[derive(PartialEq, Eq, Debug)]
pub struct BasicCredentials {
    pub user: String,
    pub password: String,
}

impl BasicCredentials {
    pub fn from(user: &str, password: &str) -> BasicCredentials {
        BasicCredentials {
            user: user.to_string(),
            password: password.to_string(),
        }
    }
}

impl TryFrom<&str> for BasicCredentials {
    type Error = &'static str;

    fn try_from(
        value: &str
    ) -> Result<Self, Self::Error> {
        let pair = base64::engine::general_purpose::STANDARD.decode(value)
            .map_err(|_err| "Invalid base base64 in authorization header")?;

        let pair = String::from_utf8(pair).unwrap();

        if !pair.contains(":") {
            return Err("Invalid user-id/password pair: data must be separated by ':'");
        }

        let credentials: Vec<&str> = pair.split(":").collect();

        if credentials.len() != 2 {
            return Err("Invalid user-id/password pair: pair length must be equal 2");
        }

        Ok(BasicCredentials::from(
            credentials.get(0).unwrap(),
            credentials.get(1).unwrap()))
    }
}

#[derive(Debug)]
pub struct InvalidAuthMethod;

#[derive(Debug)]
pub struct InvalidCredentials;

impl reject::Reject for InvalidAuthMethod {}

impl reject::Reject for InvalidCredentials {}


pub fn basic_auth(
    password: String,
) -> impl Filter<Extract=(), Error=Rejection> + Clone {
    static BASIC_AUTH_RE: Lazy<Regex> = Lazy::new(|| Regex::new("Basic ([\\S]+)").unwrap());

    let password = Arc::new(password);

    let header = warp::header::<String>("authorization")
        .and_then(move |header: String| {
            let password = password.clone();
            async move {
                if let Some(captures) = BASIC_AUTH_RE.captures(&header) {
                    if let Some(credentials) = captures.get(1) {
                        let other = BasicCredentials::try_from(credentials.as_str()).unwrap();

                        if *password.clone() != other.password {
                            return Err(reject::custom(InvalidCredentials));
                        }

                        return Ok(());
                    }
                }
                return Err(reject::custom(InvalidAuthMethod));
            }
        });

    header.untuple_one()
}