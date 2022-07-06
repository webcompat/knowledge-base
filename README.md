# The Web Compatibility Knowledge Base

This repository contains a subset of the Web Compatibility teams' knowledge about real-world site breakage. With this collection, we hope to

- ease some diagnosis tasks by offering a searchable collection of already known breakage and their symptoms,
- inform Firefox engineering prioritization,
- share knowledge between members of the Web Compatibility team.

Please note that this is still an early **work in progress**, and everything in this repo can and will change. Therefore, you should not rely on the data without consulting the WebCompat team first!

## Additional information

- [Criteria for the `severity` and `user_base_impact` fields](./docs/severity-and-impact.md).
- [Script to generate a yml file for a provided bugzilla bug](./docs/generate-yml.md)

## Tooling

The `kbcheck` tool is designed to help interact with the knowledge base. It's written in [Rust](https://www.rust-lang.org/) and is most easily used by running `cargo run` in the checkout. The following commands are available:

- `tags`: Print a list of tags currently used in knowledge base entries.
- `validate`: Validate the knowledge base entries against the schema and additional lint rules.

## License

[Creative Commons Public Domain Dedication (CC0 1.0)](https://creativecommons.org/publicdomain/zero/1.0/).
