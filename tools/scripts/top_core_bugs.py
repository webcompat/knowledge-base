import html
import logging
import urllib
from typing import Any, Mapping, MutableMapping, Optional

import requests

from helpers import github, issue

Bug = Mapping[str, Any]

logging.basicConfig()
logger = logging.getLogger()
logger.setLevel(logging.DEBUG)


def get_kb_bugs() -> set[int]:
    resp = requests.get("https://bugzilla.mozilla.org/rest/bug?product=Web+Compatibility&Component=Knowledge+Base&include_fields=id")
    resp.raise_for_status()
    return {item["id"] for item in resp.json()["bugs"]}


def get_bug_list() -> list[Bug]:
    resp = requests.get("https://bugzilla.mozilla.org/rest/bug?f1=see_also&f2=component&n2=1&o1=anywordssubstr&o2=anywordssubstr&classification=Components&product=Core&resolution=---&v1=webcompat&v2=Privacy&limit=0&include_fields=id,component,see_also,priority,severity,summary,blocks")
    resp.raise_for_status()
    return resp.json()["bugs"]


def webcompat_link(see_also: str) -> Optional[str]:
    try:
        url = urllib.parse.urlparse(see_also)
    except Exception:
        return None
    if url.hostname == "webcompat.com":
        bug_id = url.path.rsplit("/", 1)[1]
        return f"https://github.com/webcompat/web-bugs/issues/{bug_id}"
    if url.hostname == "github.com" and url.path.startswith("/webcompat/web-bugs/issues/"):
        return f"{url.scheme}://{url.hostname}{url.path}"
    return None


def bug_link(dest_id: int) -> str:
    return f"<a href=https://bugzilla.mozilla.org/show_bug.cgi?id={dest_id}>{dest_id}</a>"


def html_list(list_items: list[str]) -> str:
    if list_items:
        return "<ul>" + "".join(f"<li>{item}" for item in list_items) + "</ul>"
    return ""


def format_as_html(bugs: list[Bug]) -> str:
    rv = ["<!doctype html>",
          "<title>Core Bugs with WebCompat Links</title>",
          "<h1>Core Bugs with WebCompat Links</h1>",
          f"<p>Found {len(bugs)} bugs"]
    for offset in range(4):
        rv.extend([f"<h2 id=group{offset + 1}>Group {offset + 1}</h2>",
                   "<table>",
                   "<tr><th>id<th>Summary<th>Component<th>Severity<th>Webcompat Links<th>Issue Priority Score<th>Max Issue Severity<th>KB Entries"])
        for bug in bugs[offset::4]:
            kb_links = html_list([bug_link(item) for item in bug["kb_entries"]])
            webcompat_links = []
            for (link, issue) in zip(bug["webcompat_links"], bug["issues"]):
                priority, severity = get_severity_priority(issue)
                webcompat_links.append(f"<a href={link}>{link}</a> [P{priority} S{severity}]")
                webcompat_details = f"<details>{html_list(webcompat_links)}</details>" if webcompat_links else ""
            rv.append(f"<tr><td>{bug_link(bug['id'])}<td>{html.escape(bug['summary'])}<td>{bug['component']}<td>{bug['severity']}<td>{len(bug['webcompat_links'])}{webcompat_details}<td>{bug['score'][0]}<td>{-bug['score'][1]}<td>{kb_links}")
        rv.append("</table>")
    return "\n".join(rv)


def bug_score(bug):
    bug_severity = int(bug["severity"][1]) if bug["severity"][0] == "S" else 5
    issue_score = []
    severity_score = 4
    priority_score = 0
    for issue in bug["issues"]:
        issue_priority, issue_severity = get_severity_priority(issue)
        priority_score += 10 ** (4 - issue_priority)
        severity_score = min(severity_score, issue_severity)
    return (priority_score, -severity_score, len(bug["webcompat_links"]), -bug_severity)


severity_map = {"critical": 2, "important": 3, "minor": 4}


def get_severity_priority(gh_issue):
    priority = 4
    severity = 4
    for label in gh_issue.labels:
        if label.startswith("diagnosis-priority-"):
            priority = int(label[-1])
        elif label.startswith("severity"):
            severity = severity_map[label.rsplit("-", 1)[1]]
    return (priority, severity)


def main():
    bug_cache = {}
    kb_bugs = get_kb_bugs()
    bugs = get_bug_list()
    bugs = [bug for bug in bugs if "Layout" in bug["component"]]
    gh_client = github.GitHub()
    for bug in bugs:
        bug["webcompat_links"] = []
        bug["issues"] = []
        for item in bug.get("see_also", []):
            link = webcompat_link(item)
            if link is not None:
                bug["webcompat_links"].append(link)
                issue_id = link.rsplit("/", 1)[1]
                bug["issues"].append(issue.WebcompatIssue.from_dict(gh_client.fetch_issue_by_number("webcompat", "web-bugs", issue_id, timeline=False)))
        bug["kb_entries"] = [item for item in bug["blocks"] if item in kb_bugs]
        bug["score"] = bug_score(bug)
    bugs.sort(key=lambda bug: bug["score"], reverse=True)
    print(format_as_html(bugs))


if __name__ == "__main__":
    main()
