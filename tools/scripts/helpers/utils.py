import re
from datetime import datetime


def extract_url(issue_body: str) -> str:
    """Extract the URL for an issue from WebCompat.

    URL in webcompat.com bugs follow this pattern:
    **URL**: https://example.com/foobar
    """
    url_pattern = re.compile(r"\*\*URL\*\*\: (.+)")
    url_match = re.search(url_pattern, issue_body)
    if url_match:
        url = url_match.group(1).strip()
        if not url.startswith(("http://", "https://")):
            url = "http://%s" % url
    else:
        url = ""
    return url


def to_datetime(date_str: str) -> datetime:
    """Convert date to datetime object.

    date_str: 2015-07-28T09:25:03Z
    """
    normalized_date_str = date_str.replace("Z", "+00:00")
    return datetime.fromisoformat(normalized_date_str)


def extract_issue_number(url_str: str) -> str:
    num = re.findall('[0-9]+', url_str)
    return num[0]
