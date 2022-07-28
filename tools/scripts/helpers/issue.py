from dataclasses import dataclass
from typing import List

from helpers.github import GitHub
from helpers.utils import extract_url, to_datetime, extract_issue_number

WEBCOMPAT_URL = "https://webcompat.com/issues/"

BROWSER_MAP = {
    "browser-fenix": "mobile",
    "browser-firefox-mobile": "mobile",
    "browser-focus-geckoview": "mobile",
    "browser-android-components": "mobile",
    "browser-firefox-ios": "mobile",
    "browser-firefox": "desktop"
}

RESOLVED_MAP = {
    "fixed": "site_fixed",
    "worksforme": "site_changed",
    "invalid": "site_changed"
}

SEVERITY_MAP = {
    "severity-minor": "minor_visual",
    "severity-important":
        "Suggested impact is `feature_broken` or `significant_visual` as issue has severity-important label.",
    "severity-critical": "Suggested impact is `site_broken` or `feature_broken` as issue has severity-critical label.",
    "type-unsupported": "unsupported_message"
}

TRACKED_MILESTONES = [
    "needsdiagnosis",
    "duplicate",
    "moved",
    "contactready",
    "needscontact",
    "sitewait"
]


@dataclass
class WebcompatIssue:
    body: str
    breakage_url: str
    number: int
    title: str
    labels: List[str]
    milestone: str
    timeline: List[dict]

    @classmethod
    def from_dict(cls, issue: dict):
        original_labels = issue.get("labels", [])
        labels = [label["name"] for label in original_labels]
        issue_body = issue["body"]
        breakage_url = extract_url(issue_body)

        milestone = ""
        if issue.get("milestone"):
            milestone = issue["milestone"]["title"]

        return cls(body=issue_body,
                   breakage_url=breakage_url,
                   number=issue["number"],
                   title=issue["title"],
                   labels=labels,
                   milestone=milestone,
                   timeline=issue["timeline"])

    def build_canonical_url(self) -> str:
        return WEBCOMPAT_URL + str(self.number)

    def get_platform(self) -> list:
        browser = []
        for label in self.labels:
            if label in BROWSER_MAP:
                browser.append(BROWSER_MAP[label])

        if len(browser) >= 2:
            browser = ['all']

        return browser

    def get_last_reproduced(self) -> str:
        if self.milestone in TRACKED_MILESTONES:
            for timeline_item in self.timeline:
                if (timeline_item["event"] == "milestoned"
                        and timeline_item["milestone"]["title"] == self.milestone):
                    return to_datetime(timeline_item["created_at"]).strftime('%Y-%m-%d')

        return ""

    def estimate_impact(self) -> str:
        if "type-unsupported" in self.labels:
            return SEVERITY_MAP["type-unsupported"]

        severity_label = [label for label in self.labels if label.startswith('severity-')]

        if severity_label:
            return SEVERITY_MAP[severity_label[0]]

        return "Can't estimate impact, please enter it manually."

    def generate_report(self) -> dict:
        report = {
            "url": self.build_canonical_url(),
            "site": self.breakage_url,
            "platform": self.get_platform()
        }

        if self.milestone in RESOLVED_MAP.keys():
            report["resolution"] = RESOLVED_MAP[self.milestone]

        if "sitepatch-applied" in self.labels:
            report["intervention"] = "Intervention is detected for this issue. Please locate it manually."

        report["impact"] = self.estimate_impact()
        report["affects_users"] = f"Check the issue for more details: {self.title}."

        reproduced_date = self.get_last_reproduced()
        if reproduced_date:
            report["last_reproduced"] = reproduced_date

        return report


class Issue:
    def __init__(self, url) -> None:
        self.url = url

        if "webcompat" in url:
            self.type = "webcompat"
        elif "mozilla-mobile" in url:
            self.type = "fenix"

    def generate_report(self) -> dict:
        report = {
            "url": self.url
        }

        if self.type == "webcompat":
            issue_number = extract_issue_number(self.url)
            github = GitHub()
            issue_data = github.fetch_issue_by_number(
                "webcompat",
                "web-bugs",
                issue_number
            )
            wc_issue = WebcompatIssue.from_dict(issue_data)
            report = wc_issue.generate_report()

        return report
