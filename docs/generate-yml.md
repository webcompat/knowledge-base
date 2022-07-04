# Script to generate a yml file for a provided bugzilla bug

The script will generate a yml file in the `data` directory and prefill title, breakage reports from webcompat and fenix repository, as well as
platform links (the original bugzilla bug and any others from bugs.chromium.org or bugs.webkit.org).

It can be run the following way:

```
python3 -m venv env
source env/bin/activate
pip install -r requirements.txt
python3 ./tools/scripts/generate.py --bug_id=<bug_id>
```