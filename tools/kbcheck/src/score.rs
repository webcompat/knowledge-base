use crate::data;
use crate::entry::{Breakage, Entry, Platform};
use crate::tranco::TrancoData;
use std::path::{Path, PathBuf};
use url::Url;

fn breakage_score(breakage: &Breakage) -> f64 {
    // First estimate the impcat of the issue
    let mut score = match breakage.impact {
        crate::entry::Impact::SiteBroken => 100.,
        crate::entry::Impact::WorkflowBroken => 60.,
        crate::entry::Impact::FeatureBroken => 30.,
        crate::entry::Impact::SignificantVisual => 20.,
        crate::entry::Impact::MinorVisual => 1.,
        crate::entry::Impact::UnsupportedMessage => 20.,
    };

    // Reduced impact for issues that are resolved or have an intervention
    if breakage.resolution.is_some() {
        score *= 0.05;
    } else if breakage.intervention.is_some() {
        score *= 0.1;
    }

    // Now adjust for the expected number of users who will encounter the issue
    // according to platform, or manual asssessment
    let platform_modifier: f64 = breakage
        .platforms()
        .iter()
        .map(|platform| match platform {
            Platform::Android => 0.2,
            Platform::Windows => 0.6,
            Platform::Macos => 0.1,
            Platform::Linux => 0.1,
        })
        .sum();
    score *= platform_modifier;

    score *= match breakage.affects_users {
        crate::entry::AffectsUsers::All => 1.0,
        crate::entry::AffectsUsers::Some => 0.3,
        crate::entry::AffectsUsers::Few => 0.1,
    };

    score
}

fn site_impact(tranco_data: &TrancoData, url: &Url) -> f64 {
    let domain = match url.host_str() {
        Some(domain) => domain,
        None => return 0.,
    };

    let (mut matched, ranking) = match tranco_data.get_ranking(domain) {
        Some((matched, ranking)) => (matched, ranking),
        None => return 0.01,
    };
    matched.reverse();
    let matched_domain = matched.join(".");

    // This is not based on anything particuarly principled
    let mut impact = match ranking {
        0..=1000 => 1.,
        1001..=10_000 => 0.5,
        10_001..=100_000 => 0.2,
        _ => 0.1,
    };
    // Arbitarily decrease the score for subdomains
    if matched_domain != domain {
        impact *= 0.5;
    }
    impact
}

fn score_references(tranco_data: &TrancoData, entry: &Entry) -> Option<f64> {
    let score = entry
        .references
        .breakage
        .iter()
        .map(|breakage| match breakage {
            crate::entry::BreakageItem::Url(_) => 0.,
            crate::entry::BreakageItem::Breakage(breakage) => {
                breakage_score(breakage) * site_impact(tranco_data, &breakage.site)
            }
        })
        .sum();
    if score == 0. {
        None
    } else {
        Some(score)
    }
}

pub fn entries_by_score(
    root_path: &Path,
    data_paths: &[&Path],
    tranco_data: &TrancoData,
) -> Result<Vec<(PathBuf, Entry, f64)>, data::DataError> {
    let mut scores = Vec::new();
    let entries = if data_paths.is_empty() {
        data::load_all(root_path, false)?
    } else {
        data::load_paths(root_path, data_paths)?
    };
    for (path, entry) in entries.into_iter() {
        if let Some(score) = score_references(tranco_data, &entry) {
            scores.push((path, entry, score));
        } else {
            println!("Not scoring: {}", entry.title);
        }
    }
    scores.sort_by(|&(_, _, score_a), &(_, _, score_b)| {
        score_b.partial_cmp(&score_a).expect("Got a NaN score")
    });
    Ok(scores)
}
