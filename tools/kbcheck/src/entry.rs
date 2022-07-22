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
    pub breakage: Vec<BreakageItem>,
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
#[serde(untagged)]
pub enum BreakageItem {
    Url(Url),
    Breakage(Breakage),
}

impl BreakageItem {
    pub fn url(&self) -> &Url {
        match self {
            BreakageItem::Url(url) => url,
            BreakageItem::Breakage(item) => &item.url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Breakage {
    url: Url,
    site: Url,
    platform: Vec<Platform>,
    last_reproduced: Option<String>, // Date
    intervention: Option<Url>,
    impact: Impact,
    affects_users: AffectsUsers,
    resolution: Option<BreakageResolution>,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    All,
    Desktop,
    Mobile,
    Windows,
    Macos,
    Linux,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Impact {
    SiteBroken,
    FeatureBroken,
    SignificantVisual,
    MinorVisual,
    UnsupportedMessage,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AffectsUsers {
    All,
    Some,
    Few,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakageResolution {
    SiteChanged,
    SiteFixed,
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
