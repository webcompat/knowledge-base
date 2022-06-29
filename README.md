# The Web Compatibility Knowledge Base

This repository contains a subset of the Web Compatibility teams' knowledge about real-world site breakage. With this collection, we hope to

- ease some diagnosis tasks by offering a searchable collection of already known breakage and their symptoms,
- inform Firefox engineering prioritization,
- share knowledge between members of the Web Compatibility team.

Please note that this is still an early **work in progress**, and everything in this repo can and will change. Therefore, you should not rely on the data without consulting the WebCompat team first!

## Additional information

- [Criteria for the `severity` and `user_base_impact` fields](./docs/severity-and-impact.md).

## Script to create yml file for a given bugzilla bug

The script will create a file in the `data` directory and prefill title, breakage reports from webcompat and fenix repository and
platform links (the original bugzilla bug and any others from bugs.chromium.org or bugs.webkit.org).

It can be run the following way:

```
python3 -m venv env
source env/bin/activate
pip install -r requirements.txt
python3 ./tools/scripts/generate.py --bug_id=<bug_id>
```

## License

[Creative Commons Public Domain Dedication (CC0 1.0)](https://creativecommons.org/publicdomain/zero/1.0/).
