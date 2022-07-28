import argparse
import ruamel.yaml as yaml
import os

from logging import INFO, basicConfig, getLogger
from pathlib import Path
from slugify import slugify
from urllib.parse import urlparse

from helpers.issue import Issue
from helpers.bugzilla import fetch_bug_by_id

basicConfig(level=INFO)
logger = getLogger("generate_script")

cwd = os.getcwd()
DATA_PATH = os.path.join(cwd, 'data')

BUGZILLA_URL = "https://bugzilla.mozilla.org/show_bug.cgi?id="

BREAKAGE_STR = ["webcompat", "mozilla-mobile"]
PLATFORM_STR = ["bugs.chromium.org", "bugs.webkit.org"]

YAML_FILE_EXT = ("*.yaml", "*.yml")


def generate_breakage_report(issue_url: str) -> dict:
    issue = Issue(issue_url)
    report = issue.generate_report()
    return report


def split_see_also_by_type(see_also_list: list, bug_id: str) -> tuple:
    breakage = []
    platform = [BUGZILLA_URL + bug_id]

    for item in see_also_list:
        if any(bs in item for bs in BREAKAGE_STR):
            breakage.append(generate_breakage_report(item))

        if any(ps in item for ps in PLATFORM_STR):
            platform.append(item)

    return breakage, platform


def build_obj(bug_id: str) -> dict:
    bug = fetch_bug_by_id(bug_id)
    breakage, platform = split_see_also_by_type(bug["see_also"], bug_id)
    data = {
        "title": bug["summary"],
        "references": {
            "breakage": breakage,
            "platform_issues": platform
        }
    }

    return data


def check_for_duplicates(bug_id: str) -> list:
    folder = Path(DATA_PATH)
    files = [f for f in folder.iterdir() if any(f.match(p) for p in YAML_FILE_EXT)]
    duplicates = []

    for file_path in files:
        with open(file_path, 'r') as stream:
            try:
                contents = yaml.safe_load(stream)
                # Check references -> platform_issues list to see if there is a duplicate
                if "references" in contents and "platform_issues" in contents["references"]:
                    for platform_url in contents["references"]["platform_issues"]:
                        parsed = urlparse(platform_url)
                        url = parsed._replace(fragment="").geturl()

                        if url == BUGZILLA_URL + bug_id:
                            duplicates.append(str(file_path))

            except yaml.YAMLError as exc:
                logger.error(exc)

    return duplicates


def build_yml(bug_id: str) -> None:
    duplicates = check_for_duplicates(bug_id)

    if duplicates:
        logger.error(
            f"A duplicate bug url has been found in {','.join(duplicates)}"
        )
        return

    data = build_obj(bug_id)
    title = data["title"]

    filename = slugify(title, max_length=40, word_boundary=True, save_order=True)
    path = f"{DATA_PATH}/{filename}.yml"

    if os.path.exists(path):
        logger.error(f"A file with the same name already exists: `{path}`, please edit it instead.")
        return

    y = yaml.YAML()
    y.indent(mapping=2, sequence=4, offset=2)

    with open(path, 'w') as f:
        y.dump(data, f)
        logger.info(f"{filename}.yml file was created for bug {bug_id} ({title})")


def main() -> None:
    description = "Prefill yaml file with bugzilla bug data for knowledge base."
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument(
        "--bug_id",
        help="Bugzilla bug id.",
        type=str,
        required=True,
    )

    args = parser.parse_args()
    build_yml(args.bug_id)


if __name__ == "__main__":
    main()
