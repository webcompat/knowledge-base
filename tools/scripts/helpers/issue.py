from dataclasses import dataclass
from typing import List, Dict

from helpers.github import GitHub
from helpers.utils import extract_url, to_datetime, extract_issue_number

WEBCOMPAT_URL = "https://webcompat.com/issues/"
MISSING_PLACEHOLDER = "[missing]"

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
    "severity-important": MISSING_PLACEHOLDER,
    "severity-critical": MISSING_PLACEHOLDER,
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

    def build_milestone_dates_map(self) -> Dict[str, dict]:
        """Builds milestones->dates map for further processing.

        Returns a dictionary for each milestone event with
        dates it has been milestoned with or demilestoned.

        Example structure:
        {
            'needsdiagnosis': {
                'milestoned': '2020-09-21T14:52:00Z',
                'milestoned_from': 'needstriage',
                'demilestoned': '2021-04-27T05:59:20Z'
            }
        }
        """
        milestones_dates = {}  # type: Dict[str, dict]
        actions = ["milestoned", "demilestoned"]

        for index, timeline_item in enumerate(self.timeline):
            if timeline_item["event"] in actions:
                milestone = timeline_item["milestone"]["title"]
                action = timeline_item["event"]

                if milestone not in milestones_dates:
                    milestones_dates[milestone] = {}

                milestones_dates[milestone][action] = timeline_item["created_at"]

                if action == "milestoned" and self.timeline[index-1]["event"] == "demilestoned":
                    milestones_dates[milestone]["milestoned_from"] = self.timeline[index-1]["milestone"]["title"]

        return milestones_dates

    def last_reproduced_for_tracked(self, milestones_dates: dict) -> str:
        """Attempt to find last reproduced date for issues with "active" milestones."""

        if "milestoned" in milestones_dates[self.milestone]:
            return to_datetime(milestones_dates[self.milestone]["milestoned"]).strftime('%Y-%m-%d')

        return MISSING_PLACEHOLDER

    def last_reproduced_for_resolved(self, milestones_dates: dict) -> str:
        """Attempt to find last reproduced date for resolved issues.

        Finds a previous milestone and if it's considered "active",
        returns the date the issue has been moved to that milestone.
        """

        if "milestoned_from" in milestones_dates[self.milestone]:
            previous_milestone = milestones_dates[self.milestone]["milestoned_from"]

            if previous_milestone in TRACKED_MILESTONES:
                if (previous_milestone in milestones_dates
                        and "milestoned" in milestones_dates[previous_milestone]):
                    return to_datetime(milestones_dates[previous_milestone]["milestoned"]).strftime('%Y-%m-%d')

        return MISSING_PLACEHOLDER

    def get_last_reproduced(self) -> str:
        milestones_dates = self.build_milestone_dates_map()

        if self.milestone not in milestones_dates:
            return MISSING_PLACEHOLDER

        if self.milestone in TRACKED_MILESTONES:
            return self.last_reproduced_for_tracked(milestones_dates)
        elif self.milestone in RESOLVED_MAP.keys():
            return self.last_reproduced_for_resolved(milestones_dates)

        return MISSING_PLACEHOLDER

    def estimate_impact(self) -> str:
        if "type-unsupported" in self.labels:
            return SEVERITY_MAP["type-unsupported"]

        severity_label = [label for label in self.labels if label.startswith('severity-')]

        if severity_label:
            return SEVERITY_MAP[severity_label[0]]

        return MISSING_PLACEHOLDER

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
        report["affects_users"] = MISSING_PLACEHOLDER
        report["last_reproduced"] = self.get_last_reproduced()

        return report


class Issue:
    def __init__(self, url) -> None:
        self.url = url

        if "webcompat" in url:
            self.type = "webcompat"
        elif "mozilla-mobile" in url:
            self.type = "fenix"
        else:
            self.type = "other"

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
