# CHANGELOG for `humanlog`
This file keeps track of changes to the `humanlog` crate.

It makes use of [semantic versioning](https://semver.org). As such, any breaking changes are indicated with **(BREAKING)**.


## v0.2.0 - 2024-09-08
This release sees a change in licensing to Apache 2.0. See [LICENSE](./LICENSE) for more details.

### Fixed
- Removed `atty` as a dependency because it is unmaintained. Instead relying on [`std::io::IsTerminal::is_terminal()`](https://doc.rust-lang.org/stable/std/io/trait.IsTerminal.html#tymethod.is_terminal) from the standard library to discover if coloration needs to be applied.


## v0.1.0 - 2023-05-17
Initial release!
