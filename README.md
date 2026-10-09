# http-kit

HTTP request and response helpers for Rust.

GitHub Actions runs the checks and builds documentation on pull requests and
pushes to `main`.

Format, lint, and test the workspace (requires stable Rust, rustfmt, Clippy, and jq):

```sh
./scripts/check.sh
```

Enable the pre-push hook after cloning:

```sh
git config --local core.hooksPath .githooks
```
