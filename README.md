# http-kit

Rust helpers for request validation, pagination metadata, API responses, and
field selection. Your application fetches data, creates cursors, and builds
the selected response fields.

## Workspace

| Crate                                        | Purpose                                                           |
| -------------------------------------------- | ----------------------------------------------------------------- |
| [http-kit](http-kit/README.md)               | Runtime types, Axum extractors, and optional integrations         |
| [http-kit-macros](http-kit-macros/README.md) | Query attributes and response derives                             |
| `http-kit-macros-tests`                      | Integration tests, compile-fail tests, and documentation examples |

## Usage

These workspace packages have `publish = false`. Use a local path dependency;
adjust the path for your project:

```toml
[dependencies]
http-kit = { path = "../http-kit/http-kit", features = ["axum"] }
serde = { version = "1", features = ["derive"] }
valid8r = "0.2.2"
```

`response` is enabled by default. Ordering and pagination metadata are always
available. See the [feature table](http-kit/README.md#features) for other options.

## Development

Install stable Rust, rustfmt, Clippy, and jq. Check without changing formatting:

```sh
./scripts/check.sh --check
```

To format first, then run the same checks:

```sh
./scripts/check.sh
```

The gate runs linting, tests, documentation examples, documentation builds,
and isolated feature checks. GitHub Actions runs it on pull requests and
pushes to `main`.

Optional pre-push hook:

```sh
git config --local core.hooksPath .githooks
```
