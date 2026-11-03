# Contributing

Use the stable Rust toolchain. Keep changes focused, update the English and Chinese user documentation when behavior changes, and add regression tests for bugs.

Before submitting a pull request, run:

```sh
cargo fmt --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
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

## Releases

1. Update the version in `Cargo.toml`, refresh `Cargo.lock`, and update `docs/RELEASE.md` for users. The executable reads its version from Cargo metadata.
2. Merge the reviewed changes after CI passes.
3. Push the matching version tag, for example `v0.1.1` for version `0.1.1`.
4. Check the Build and Release workflow and inspect the resulting archives.

Release builds use the committed lockfile and an explicit Rust target. Each archive includes the executable, README, user documentation, and the project and font licenses. Only the publication job has permission to write repository contents.
