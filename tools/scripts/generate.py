import requests
import argparse
import yaml
import os
from pathlib import Path
from slugify import slugify

cwd = os.getcwd()
DATA_PATH = os.path.join(cwd, 'data')

BUGZILLA_URL = "https://bugzilla.mozilla.org/show_bug.cgi?id="
BUGZILLA_API = "https://bugzilla.mozilla.org/rest/bug/"

BREAKAGE_STR = ["webcompat", "mozilla-mobile"]
PLATFORM_STR = ["bugs.chromium.org", "bugs.webkit.org"]

YAML_FILE_EXT = ("*.yaml", "*.yml")


def fetch_bug_by_id(bug_id) -> dict:
    url = BUGZILLA_API + bug_id
    params = {"include_fields": "summary,see_also"}
    response = requests.get(url, params=params)
    response.raise_for_status()
    result = response.json()
    return result["bugs"][0]


def split_see_also_by_type(see_also_list, bug_id) -> list:
    lists = {
        "breakage": [f"{BUGZILLA_URL}{bug_id}#c0"],
        "platform": [BUGZILLA_URL + bug_id]
    }

    for item in see_also_list:
        if any(bs in item for bs in BREAKAGE_STR):
            lists["breakage"].append(item)

        if any(ps in item for ps in PLATFORM_STR):
            lists["platform"].append(item)

    return lists


def build_obj(bug_id) -> dict:
    bug = fetch_bug_by_id(bug_id)
    lists = split_see_also_by_type(bug["see_also"], bug_id)
    data = {
        "title": bug["summary"],
        "references": {
            "breakage": lists["breakage"],
            "platform_issues": lists["platform"]
        }
    }

    return data


def check_existing_entries(bug_id) -> list:
    folder = Path(DATA_PATH)
    files = [f for f in folder.iterdir() if any(f.match(p) for p in YAML_FILE_EXT)]

    for file in files:
        with open(file, 'r') as stream:
            try:
                contents = yaml.safe_load(stream)
                # Check the first item in references -> platform_issues list to see if there is a match
                if "references" in contents and "platform_issues" in contents["references"]:
                    if contents["references"]["platform_issues"][0] == BUGZILLA_URL + bug_id:
                        return file

            except yaml.YAMLError as exc:
                print(exc)


def build_yml(bug_id) -> None:
    existing_entry = check_existing_entries(bug_id)

    if existing_entry:
        print(f"A knowledge base entry for this bug has been found. Please see `{existing_entry}` for more details.")
        return

    data = build_obj(bug_id)
    title = data["title"]

    filename = slugify(title, max_length=40, word_boundary=True, save_order=True)
    path = f"{DATA_PATH}/{filename}.yml"

    if os.path.exists(path):
        print(f"A file with the same name already exists: `{path}`, please edit it instead.")
        return

    with open(path, 'w') as f:
        yaml.dump(data, f, sort_keys=False, default_flow_style=False)
        print(f"{filename}.yml file was created for bug {bug_id} ({title})")


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
