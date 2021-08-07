use std::string::FromUtf8Error;
use thiserror::Error;
use warp::reject::Reject;

pub type MultihookResult<T> = Result<T, MultihookError>;

#[derive(Error, Debug)]
pub enum MultihookError {
    #[error(transparent)]
    Warp(#[from] warp::Error),

    #[error("Failed to parse body as utf8 string {0}")]
    UTF8Error(#[from] FromUtf8Error),

    #[error("Unknown endpoint")]
    UnknownEndpoint,
}

impl Reject for MultihookError {}
