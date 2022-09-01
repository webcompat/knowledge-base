import logging
import os
import requests

from dotenv import load_dotenv, find_dotenv
load_dotenv(find_dotenv())

logger = logging.getLogger(__name__)


class GitHub:
    def get_headers(self) -> dict:
        token = os.environ.get("GITHUB_TOKEN")

        if not token:
            return {}

        return {"Authorization": "token {}".format(token)}

    def fetch_timeline(self, timeline_url: str) -> list:
        params = {
            "per_page": 100,
        }
        response = requests.get(timeline_url, params=params, headers=self.get_headers())
        response.raise_for_status()
        timeline_raw = response.json()
        return timeline_raw

    def fetch_issue_by_number(self, owner: str, repo: str, issue_number: str) -> dict:
        url = f"https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}"
        logger.info(f"Fetching issue: {url}")
        response = requests.get(url, headers=self.get_headers())
        response.raise_for_status()
        issue_raw = response.json()

        timeline = self.fetch_timeline(url + "/timeline")
        issue_raw.update({"timeline": timeline})

        return issue_raw
