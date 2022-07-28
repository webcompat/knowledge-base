import requests

BUGZILLA_API = "https://bugzilla.mozilla.org/rest/bug/"


def fetch_bug_by_id(bug_id: str) -> dict:
    url = BUGZILLA_API + bug_id
    params = {"include_fields": "summary,see_also"}
    response = requests.get(url, params=params)
    response.raise_for_status()
    result = response.json()
    return result["bugs"][0]
