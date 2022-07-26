use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserBaseImpact {
    Large,
    Medium,
    Small,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Solutions {
    #[serde(default)]
    pub interventions: Vec<Url>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub workarounds: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct References {
    #[serde(default)]
    pub breakage: Vec<Url>,
    #[serde(default)]
    pub platform_issues: Vec<Url>,
    #[serde(default)]
    pub telemetry: Vec<Url>,
    #[serde(default)]
    pub testcases: Vec<String>, // TODO: how to handle relative URLs?
    #[serde(default)]
    pub standards_positions: Vec<Url>,
    #[serde(default)]
    pub standards_discussions: Vec<Url>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub title: String,
    pub severity: Severity,
    pub user_base_impact: Option<UserBaseImpact>,
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub symptoms: Vec<String>,
    #[serde(default)]
    pub console_messages: Vec<String>,
    #[serde(default)]
    pub solutions: Solutions,
    #[serde(default)]
    pub references: References,
}
