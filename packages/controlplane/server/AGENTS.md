# Control Plane Rust Server - Agent Guide

This service follows the feature-first layout used by `packages/unitycatalog/server`.

- Keep process wiring in `src/main.rs` and shared dependencies in `src/app_state.rs`.
- Put HTTP features under `src/features/<feature>/`.
- Put infrastructure adapters under `src/infrastructure/`.
- Edit SQL in `queries/` and `schema.sql`, then regenerate `cornucopia/` with `cornucopia schema schema.sql`.
- Never hand-edit generated files under `cornucopia/`.
- Use `thiserror` for request/application errors, `anyhow` for startup failures, and `tracing` for observability.
- Run `cargo fmt`, `cargo check`, and `cargo test` before submitting changes.

