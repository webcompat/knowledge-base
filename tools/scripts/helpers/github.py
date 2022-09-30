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

    def fetch_data(self, url: str, params: dict = None) -> tuple:
        response = requests.get(url, params=params, headers=self.get_headers())
        response.raise_for_status()
        data = response.json()

        return data, response.links

    def fetch_timeline(self, timeline_url: str) -> list:
        params = {
            "per_page": 100,
            "page": 1
        }

        data, response_links = self.fetch_data(
            url=timeline_url, params=params
        )

        while "next" in response_links.keys():
            next_page_timeline, response_links = self.fetch_data(
                response_links["next"]["url"]
            )
            data += next_page_timeline

        return data

    def fetch_issue_by_number(self, owner: str, repo: str, issue_number: str) -> dict:
        url = f"https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}"
        logger.info(f"Fetching issue: {url}")

        issue_data = self.fetch_data(url)[0]
        issue_data["timeline"] = self.fetch_timeline(url + "/timeline")

        return issue_data
