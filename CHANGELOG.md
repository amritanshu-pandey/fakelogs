# Changelog

All notable changes to fakelogs. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.4.0] - 2026-10-25

### Added

- `-n, --count <N>` stops after N events. For `benchmark` it replaces the default of 100000.
- `--seed <N>` makes the random values repeatable for the same seed, patterns and `--threads`. Time placeholders still come from the clock.
- `--version` works on its own.

### Changed

- The command-line parser moved from structopt to clap 4. Options are unchanged; help and error messages look slightly different.
- The Docker image pins its Rust version, caches dependency builds and includes the example configs under `/app/example`.
- `Cargo.lock` is committed, so release and Docker builds use the same dependency versions.
- The dataset submodule is fetched over HTTPS, so cloning no longer needs GitHub SSH keys.

## [0.3.0] - 2026-10-25

### Added

- Pattern weights: a pattern can be written as `pattern:` with a `weight:` to set how often it appears. Weights are exact and evenly spread; 0 turns a pattern off.
- `-p, --pattern` gives patterns on the command line, with or without a config file.
- `generate --eps N` overrides the config file's rate, and `benchmark --eps N` benchmarks one second at N events per second.

### Changed

- 31 example configs weight their patterns like real traffic.

## [0.2.0] - 2026-10-24

### Added

- 41 example configs for common log formats, from nginx and HAProxy to CloudTrail, Kubernetes, OpenTelemetry, Zeek and Suricata.
- Placeholders `{int:MIN-MAX}`, `{choice:A|B|C}`, `{uuid}`, `{ipv6}`, `{hex:N}`, `{unix}` and `{month_name}`.
- Clear config errors with the file, line and column, and warnings for likely placeholder typos.
- Multi-line and tab-separated patterns, using `\n` and `\t` in double-quoted YAML strings.

### Changed

- **Breaking:** `{day}`, `{month}`, `{hour}`, `{hour12}`, `{minute}` and `{second}` are zero padded.
- **Breaking:** unknown keys in the config file are an error.
- `eps` is optional and defaults to 10.

## [0.1.2] - 2026-10-24

### Added

- `--threads` for both `generate` and `benchmark`, sharing any EPS cap across threads.
- `{ts_rfc3339}`, the correctly spelled timestamp placeholder; `{ts_rfc3389}` still works.
- Release builds for macOS on Apple Silicon.

### Changed

- About 5x faster on one thread, from parsing patterns once and buffering output.
- The Windows release is a `.zip`.

### Fixed

- EPS caps are now accurate; they used to fall well short at higher rates.
- Each `{ipv4}` in a pattern gets its own address, and every repeated timestamp placeholder is filled in.
- Sentences and paragraphs had one word or sentence too few.
- `{email}` uses names rather than random words.
- Piping into `head` exits cleanly instead of panicking.

## [0.1.1] - 2022-09-23

### Changed

- The example config is attached to each release as `config.yaml`.

## [0.1.0] - 2022-09-17

- First release.

[Unreleased]: https://github.com/amritanshu-pandey/fakelogs/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/amritanshu-pandey/fakelogs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/amritanshu-pandey/fakelogs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/amritanshu-pandey/fakelogs/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/amritanshu-pandey/fakelogs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/amritanshu-pandey/fakelogs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/amritanshu-pandey/fakelogs/releases/tag/v0.1.0
