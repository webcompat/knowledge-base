# Script to generate a yml file for a provided bugzilla bug

The script will generate a yml file in the `data` directory and prefill title, breakage reports from webcompat and fenix repository, as well as
platform links (the original bugzilla bug and any others from bugs.chromium.org or bugs.webkit.org).

The script can be run the following way:

```
python3 -m venv env
source env/bin/activate
pip install -r requirements.txt
python3 ./tools/scripts/generate.py --create=<bug_id> --write
```
This will generate an entry file for the provided bugzilla bug id.

To add detailed breakage section to an existing entry (for example, marquee):

```
python3 ./tools/scripts/generate.py --get-breakage-details=data/marquee.yml --write
```


*Optional:*

To bypass GitHub limit of 60 requests per hour for unauthenticated users, generate a new GitHub token on https://github.com/settings/tokens
and create an .env file in the root directory with the following content:

```
GITHUB_TOKEN = '<your token>'
```