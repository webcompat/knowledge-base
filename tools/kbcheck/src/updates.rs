use crate::bugzilla;
use crate::data::load_all;
use crate::entry::Entry;
use miette::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug, Diagnostic)]
pub enum UpdateError {
    #[error(transparent)]
    DataError(#[from] crate::data::DataError),
    #[error(transparent)]
    BugzillaError(#[from] crate::bugzilla::BugzillaError),
}

pub struct Update {
    pub error: String,
    pub suggestion: String,
}

/// Extract a bugzilla bug id from a URL
fn bugzilla_id(url: &Url) -> Option<u64> {
    if url.host_str() == Some("bugzilla.mozilla.org") && url.path() == "/show_bug.cgi" {
        if let Some((_, id_str)) = url.query_pairs().find(|(name, _)| name == "id") {
            return id_str.parse().ok();
        }
    }
    None
}

/// Extract a webcompat issue number from a URL
fn webcompat_id(url: &Url) -> Option<u64> {
    if (url.host_str() == Some("github.com")
        && url.path().starts_with("/webcompat/web-bugs/issues/"))
        || (url.host_str() == Some("webcompat.com") && url.path().starts_with("/issues/"))
    {
        if let Some(segments) = url.path_segments() {
            if let Some(id) = segments.last() {
                return id.parse().ok();
            }
        }
    }
    None
}

/// Check for possible updates relative to the bugzilla bugs listed in `references.platform_issues`
fn check_bug(bugzilla: &bugzilla::Client, entry: &Entry) -> Result<Vec<Update>, UpdateError> {
    let mut updates = Vec::new();
    let platform_issue_ids = entry
        .references
        .platform_issues
        .iter()
        .filter_map(bugzilla_id);
    let mut bugs_data = BTreeMap::new();
    for bug_id in platform_issue_ids {
        let bug_data = bugzilla.get_bug(bug_id)?;
        check_is_dupe(&bug_data, &mut updates);
        bugs_data.insert(bug_id, bug_data);
    }
    check_missing_breakage(entry, &bugs_data, &mut updates);
    Ok(updates)
}

/// Check if the bug was closed as a dupe
fn check_is_dupe(bug_data: &bugzilla::Bug, updates: &mut Vec<Update>) {
    if let Some(dupe) = bug_data.dupe_of {
        updates.push(Update {
            error: format!("Bug {} closed as a duplicate of {}", bug_data.id, dupe),
            suggestion: format!(
                "Update platform_issue entry to https://bugzilla.mozilla.org/show_bug.cgi?id={}",
                dupe
            ),
        });
    }
}

/// Check for see_also links to webcompat issues that aren't in the breakage data
fn check_missing_breakage(
    entry: &Entry,
    bugs_data: &BTreeMap<u64, bugzilla::Bug>,
    updates: &mut Vec<Update>,
) {
    let entry_breakage =
        BTreeSet::from_iter(entry.references.breakage.iter().filter_map(webcompat_id));
    let mut bugs_see_also = BTreeSet::new();
    for bug_data in bugs_data.values() {
        bugs_see_also.extend(
            bug_data
                .see_also
                .iter()
                .filter_map(|url_str| Url::parse(url_str).ok().as_ref().and_then(webcompat_id)),
        )
    }
    for webcompat_id in bugs_see_also.difference(&entry_breakage) {
        updates.push(Update {
            error: "Missing see also entry from linked bug in breakage".into(),
            suggestion: format!(
                "Add https://webcompat.com/issues/{} to references.breakage",
                webcompat_id
            ),
        });
    }
}

/// Check knowledge base entries against other data sources for possible updates
pub fn check_updates(root_path: &Path) -> Result<BTreeMap<PathBuf, Vec<Update>>, UpdateError> {
    let entries = load_all(root_path)?;
    let bugzilla_client = bugzilla::Client::new(
        Url::parse("https://bugzilla.mozilla.org").expect("Failed to parse bugzilla URL"),
    );
    let mut updates = BTreeMap::new();
    for (path, entry) in entries.iter() {
        let path_updates = check_bug(&bugzilla_client, entry)?;
        if !path_updates.is_empty() {
            updates.insert(path.to_owned(), path_updates);
        }
    }
    Ok(updates)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bugzilla::Bug;
    use crate::entry::{Entry, References, Severity, Solutions};
    use std::default::Default;

    #[test]
    fn test_bugzilla_id() {
        assert_eq!(
            bugzilla_id(&Url::parse("https://bugzilla.mozilla.org/show_bug.cgi?id=1234").unwrap()),
            Some(1234)
        );
        assert_eq!(
            bugzilla_id(
                &Url::parse("https://bugzilla.mozilla.org/show_bug.cgi?id=1234&foo=bar#baz")
                    .unwrap()
            ),
            Some(1234)
        );
        assert_eq!(
            bugzilla_id(&Url::parse("https://bugzilla.mozilla.org/other.cgi?id=1234").unwrap()),
            None
        );
        assert_eq!(
            bugzilla_id(&Url::parse("https://bugzilla.mozilla.com/show_bug.cgi?id=1234").unwrap()),
            None
        );
        assert_eq!(
            bugzilla_id(&Url::parse("https://bugzilla.mozilla.org/show_bug.cgi?id=1234a").unwrap()),
            None
        );
    }

    #[test]
    fn test_webcompat_id() {
        assert_eq!(
            webcompat_id(&Url::parse("https://webcompat.com/issues/1234").unwrap()),
            Some(1234)
        );
        assert_eq!(
            webcompat_id(&Url::parse("https://github.com/webcompat/web-bugs/issues/1234").unwrap()),
            Some(1234)
        );
        assert_eq!(
            webcompat_id(&Url::parse("https://webcompat.com/issues/1234?foo=bar#baz").unwrap()),
            Some(1234)
        );
        assert_eq!(
            webcompat_id(
                &Url::parse("https://github.com/webcompat/web-bugs/issues/1234?foo=bar#baz")
                    .unwrap()
            ),
            Some(1234)
        );
        assert_eq!(
            webcompat_id(&Url::parse("https://other.webcompat.com/issues/1234").unwrap()),
            None
        );
        assert_eq!(
            webcompat_id(&Url::parse("https://github.org/webcompat/web-bugs/issues/1234").unwrap()),
            None
        );
        assert_eq!(
            webcompat_id(&Url::parse("https://webcompat.com/other/1234").unwrap()),
            None
        );
        assert_eq!(
            webcompat_id(&Url::parse("https://github.com/webcompat/web-bugs/pulls/1234").unwrap()),
            None
        );
        assert_eq!(
            webcompat_id(&Url::parse("https://github.com/other/web-bugs/issues/1234").unwrap()),
            None
        );
    }

    #[test]
    fn test_is_dupe() {
        let mut updates = Vec::new();
        let mut bug = Bug {
            ..Default::default()
        };
        check_is_dupe(&bug, &mut updates);
        assert_eq!(updates.len(), 0);
        bug.dupe_of = Some(1234);
        check_is_dupe(&bug, &mut updates);
        assert_eq!(updates.len(), 1);
    }

    #[test]
    fn test_missing_breakage() {
        let mut updates = Vec::new();
        let bug = Bug {
            see_also: vec![
                "https://webcompat.com/issues/1234".into(),
                "https://webcompat.com/issues/1235".into(),
            ],
            ..Default::default()
        };
        let entry = Entry {
            title: "test".into(),
            severity: Severity::Normal,
            user_base_impact: None,
            notes: None,
            tags: vec![],
            symptoms: vec![],
            console_messages: vec![],
            references: References {
                breakage: vec![Url::parse("https://webcompat.com/issues/1234").unwrap()],
                ..Default::default()
            },
            solutions: Solutions {
                ..Default::default()
            },
        };
        let mut bugs_data = BTreeMap::new();
        bugs_data.insert(123, bug);
        check_missing_breakage(&entry, &bugs_data, &mut updates);
        assert_eq!(updates.len(), 1);
        assert!(updates[0].suggestion.contains("1235"))
    }
}
