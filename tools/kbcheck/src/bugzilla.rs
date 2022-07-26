use miette::Diagnostic;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{self, Map, Value};
use std::io::{self, Read};
use thiserror;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Bug {
    pub actual_time: Option<f64>,
    // This is documented to return an array but b.m.o returns a string
    pub alias: Option<OneOrMany<String>>,
    pub assigned_to: String,
    pub assigned_to_detail: User,
    pub blocks: Vec<u64>,
    pub cc: Vec<String>,
    pub cc_detail: Vec<User>,
    pub classification: String,
    pub component: String,
    pub creation_time: String,
    pub creator: String,
    pub creator_detail: User,
    pub deadline: Option<String>,
    pub depends_on: Vec<u64>,
    pub dupe_of: Option<u64>,
    pub estimated_time: Option<f64>,
    pub flags: Vec<Flag>,
    pub groups: Vec<String>,
    pub id: u64,
    pub is_cc_accessible: bool,
    pub is_confirmed: bool,
    pub is_open: bool,
    pub is_creator_accessible: bool,
    pub keywords: Vec<String>,
    pub last_change_time: String, // datetime
    pub op_sys: String,
    pub platform: String,
    pub priority: String,
    pub qa_contact: String,
    pub qa_contact_detail: Option<User>,
    pub remaining_time: Option<f64>,
    pub resolution: String,
    pub see_also: Vec<String>,
    pub severity: String,
    pub status: String,
    pub summary: String,
    pub target_milestone: String,
    pub update_token: Option<String>,
    pub url: String,
    pub version: String,
    pub whiteboard: String,
    // Extra fields
    pub tags: Option<Vec<String>>,
    pub duplicates: Option<Vec<u64>>,
    // Instance-specific fields
    #[serde(flatten)]
    pub custom: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub id: u64,
    pub real_name: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Flag {
    pub id: u64,
    pub name: String,
    pub type_id: u64,
    pub creation_date: String,     // datetime
    pub modification_date: String, // datetime
    pub status: String,
    pub setter: String,
    pub requestee: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Bugs {
    pub bugs: Vec<Bug>,
    pub faults: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ErrorResponse {
    pub code: u64,
    pub documentation: String,
    pub message: String,
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum BugzillaError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error("Bugzilla API Error")]
    APIError {
        code: u64,
        documentation: String,
        message: String,
    },
    #[error("No bugs found")]
    ZarroBoogs,
}

impl From<ErrorResponse> for BugzillaError {
    fn from(err: ErrorResponse) -> Self {
        BugzillaError::APIError {
            code: err.code,
            documentation: err.documentation,
            message: err.message,
        }
    }
}

pub struct Client {
    base_url: Url,
    http_client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(base_url: Url) -> Client {
        Client {
            base_url,
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn get_bug(&self, bug_id: u64) -> Result<Bug, BugzillaError> {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Can convert bugzilla URL")
            .push("rest")
            .push("bug")
            .push(&bug_id.to_string());
        let mut resp = self.http_client.get(url).send()?;

        let mut resp_data = String::new();
        resp.read_to_string(&mut resp_data)?;

        if resp.status().is_success() {
            let data: Bugs = serde_json::from_str(&resp_data)?;
            if let Some(bug) = data.bugs.into_iter().next() {
                Ok(bug)
            } else {
                Err(BugzillaError::ZarroBoogs)
            }
        } else {
            let try_resp_err: Result<ErrorResponse, _> = serde_json::from_str(&resp_data);
            if let Ok(resp_err) = try_resp_err {
                Err(resp_err.into())
            } else {
                Err(resp
                    .error_for_status()
                    .expect_err("Response should be an error")
                    .into())
            }
        }
    }
}
