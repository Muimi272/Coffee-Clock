# Contributing

Use Rust 1.88 or newer. Keep changes focused, update the English and Chinese user documentation when behavior changes, and add regression tests for bugs.

Before submitting a pull request, run:

```sh
cargo fmt --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo audit --deny warnings
```

The CLI integration tests run the built executable without an interactive terminal. UI tests use Ratatui's test backend; timer tests advance elapsed time directly instead of sleeping.

## Continuous integration

The reusable workflow in `.github/workflows/ci.yml` checks formatting, Clippy, and tests on each supported target:

| Platform | Rust target | GitHub runner |
| --- | --- | --- |
| Linux AMD64 | `x86_64-unknown-linux-gnu` | `ubuntu-latest` |
| Windows AMD64 | `x86_64-pc-windows-msvc` | `windows-latest` |
| macOS Intel | `x86_64-apple-darwin` | `macos-15-intel` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `macos-15` |

Pull requests and pushes to `master` run this workflow. The release workflow also calls it, so a failed check prevents release builds and publication.

The matrix tests current stable Rust on every supported target. A dedicated job pins Rust 1.88 and
checks that the committed lockfile still compiles on that minimum version. Another job runs
`cargo audit --deny warnings` against `Cargo.lock`. Dependabot opens grouped weekly Cargo updates
and monthly GitHub Actions updates. Cargo groups contain only minor and patch updates; breaking
updates get individual PRs. figlet-rs 1.0.0 is deferred because of its incompatible font API and
vulnerable ZIP dependency; reassess future versions rather than disabling security checks.
Review lockfile changes and wait for CI before merging them. Keep the MSRV in the Rust setup
action's `toolchain` input, separate from the action reference, so Dependabot cannot raise it.

The project targets Rust 1.88, which was the first stable release with all language features used
here. If a dependency raises its MSRV, update `rust-version`, the CI toolchain pins, and the user
documentation together.

## Releases

1. Update the version in `Cargo.toml`, refresh `Cargo.lock`, and update `docs/RELEASE.md` for users. The executable reads its version from Cargo metadata.
2. Merge the reviewed changes after CI passes.
3. Push the matching version tag, for example `v0.1.1` for version `0.1.1`.
4. Check the Build and Release workflow and inspect the resulting archives.

Release builds use the committed lockfile and an explicit Rust target. A version tag must match `Cargo.toml`, and each extracted executable must report that exact version. Each archive includes the executable, README, changelog, sample configuration, user documentation, and the project and font licenses. Releases also include `SHA256SUMS` for the archives. Only the publication job has permission to write repository contents.
