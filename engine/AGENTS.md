# Repository Guidelines

## Project Structure & Module Organization
- Cargo workspace ties engine crates (`baml-runtime`, `baml-vm`, `baml-compiler`); shared libraries live under `baml-lib/*`.
- `cli` supplies the `baml` CLI entry point; `language_server` runs the LSP backend.
- Language bindings live in `language_client_*`, driven by templates and utilities in `generators/`.
- `.docker/` plus `e2e_tests.py` store integration fixtures; keep protocol crates (`baml-ids`, `baml-rpc`, `baml-schema-wasm`) aligned when contracts change.

## Build, Test, and Development Commands
- `cargo build --workspace` compiles every crate; add `--release` for shipping artifacts.
- `cargo check --workspace` provides a fast pre-commit validation step.
- Run `cargo fmt --workspace -- --config imports_granularity=Crate --config group_imports=StdExternalCrate` or `npm run format` to match formatting.
- `cargo clippy --workspace -- -D warnings` treats all lints as errors.
- `cargo test --workspace --lib` runs unit and integration suites; `docker build -f .docker/Dockerfile.builder -t baml_builder .` then `python3 -m pytest e2e_tests.py -s -n 10` executes the Docker e2e matrix.

## Coding Style & Naming Conventions
- Rust code uses four-space indents, `snake_case` modules/functions, `UpperCamelCase` types.
- Honor workspace import grouping and justify any `#[allow]` with comments.
- Generated clients must mirror host-language norms (TypeScript camelCase exports, Python snake_case modules); adjust templates before regenerating.
- Keep shared schemas and IDs consistent across crates when modifying serialization or RPC payloads.

## Testing Guidelines
- Place quick unit tests inline via `#[cfg(test)]`; capture cross-crate scenarios in each crate’s `tests/` directory.
- Workspace runs assume default members; toggle WASM features explicitly when required.
- The e2e suite needs Docker and `OPENAI_API_KEY`; logs land in `test_logs/<tag>` for triage.
- Flag slow or network-bound cases with `#[ignore]` and document how to re-enable them in the test body.

## Commit & Pull Request Guidelines
- Use imperative, present-tense commit subjects (e.g., "Add discriminator in unions") and keep them under 72 characters.
- Group related edits per commit and note affected crates or clients in the body when touching shared contracts.
- Pull requests explain intent, list verification commands, and link relevant Linear/GitHub issues.
- Attach screenshots or logs when changing CLI output, diagnostics, or schema surfaces.
- Tag maintainers of impacted crates or bindings before merging.

## Environment & Secrets
- Source secrets with `infisical run --env=test -- python3 -m pytest e2e_tests.py ...`; never commit raw API keys.
- Set `BAML_ALLOW_PRERELEASE=1` only while vetting prerelease behavior and remove the flag from checked-in scripts.
