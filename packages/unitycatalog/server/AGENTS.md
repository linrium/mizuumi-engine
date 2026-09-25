# Unity Catalog Rust Server - Agent Guide

This directory contains a small Rust HTTP server for Unity Catalog experiments. It is separate from the upstream Java/Armeria Unity Catalog server under `packages/unitycatalog/unitycatalog/`, but the same caution applies: catalog APIs are data- and security-sensitive, so keep changes small, explicit, and verified.

## Project layout

| Path | Purpose |
| --- | --- |
| `Cargo.toml` / `Cargo.lock` | Rust binary crate manifest and locked dependency graph. |
| `config.toml` | Local default server and PostgreSQL settings. Environment overrides use `UNITYCATALOG__...`, for example `UNITYCATALOG__SERVER__PORT=18080`. |
| `src/main.rs` | Process bootstrap: tracing, config loading, Postgres pool creation, dependency wiring, router assembly, graceful shutdown. |
| `src/app_state.rs` | Axum shared state. Store dependencies as `Arc<dyn Trait>` for dependency injection. |
| `src/config.rs` | Typed configuration structs and defaults. Keep released config names stable. |
| `src/error.rs` | Application error type and HTTP response conversion. Surface actionable errors; log internal detail with `tracing`. |
| `src/infrastructure/` | Shared infrastructure adapters such as Postgres pool construction. |
| `src/features/<feature>/` | Feature-first HTTP code. Each feature owns its routes, services, DTOs, and models. |
| `queries/` | Hand-written SQL files consumed by Cornucopia. Edit these, not generated query Rust. |
| `schema.sql` | Schema loaded by Cornucopia when generating query code. |
| `cornucopia.toml` | Cornucopia generation configuration. |
| `cornucopia/` | Generated Rust query crate (`unitycatalog_queries`). Do not hand-edit. Regenerate from `queries/` and `schema.sql`. |

## Feature module convention

Each feature should keep `mod.rs` thin and split code by role:

- `routes.rs`: Axum route registration and extractors only. Validate request input here with `axum_valid`.
- `services.rs`: Business flow and dependency calls. Define the feature service trait here when it is injected through `AppState`.
- `dtos.rs`: HTTP request/response payloads. Use `serde` and validation derives here.
- `models.rs`: Internal feature domain types. Keep these independent of Axum when possible.

Avoid repository wrappers for simple generated SQL calls. Prefer a service using the injected pool and Cornucopia functions directly unless a real persistence abstraction is needed.

## SQL and Cornucopia workflow

1. Write or update SQL in `queries/*.sql` with Cornucopia annotations such as `--! hello_message`.
2. Update `schema.sql` when generation needs database objects.
3. Regenerate from this directory:
   ```bash
   cornucopia schema schema.sql
   ```
4. Use generated functions from the `unitycatalog_queries` crate, for example:
   ```rust
   use unitycatalog_queries::queries::hello::hello_message;

   let client = pool.get().await?;
   let message = hello_message().bind(&client).one().await?;
   ```

Do not hand-edit files under `cornucopia/`; they are generated and will be overwritten.

## Style guidelines

- Prefer feature-first organization over technology-layer folders.
- Keep `main.rs` as composition/bootstrap code; move behavior into features or infrastructure modules.
- Use trait-based dependency injection at boundaries that need substitution: `Arc<dyn HealthService>`, `Arc<dyn HelloService>`, etc.
- Keep code shallow and explicit. Avoid speculative abstractions, unused public APIs, and "for later" fields.
- Use typed DTOs and `validator` constraints for request validation.
- Use `thiserror` for application errors and `anyhow` for process startup/contextual failures.
- Use `tracing` for observability; do not use `println!` in server paths.
- Do not swallow errors or return success-shaped fallbacks for infrastructure failures.
- Keep comments rare and useful; explain non-obvious decisions, not routine Rust.

## Running and validation

Run commands from `packages/unitycatalog/server/`:

```bash
cargo fmt
cargo check
cargo test
cargo run
```

Useful local smoke checks:

```bash
UNITYCATALOG__SERVER__PORT=18080 cargo run
curl --fail http://127.0.0.1:18080/health/livez
curl --fail http://127.0.0.1:18080/health/readyz
curl --fail 'http://127.0.0.1:18080/api/hello?name=test'
```

`/health/readyz` and `/api/hello` require a reachable PostgreSQL instance matching `config.toml` or the `UNITYCATALOG__POSTGRES__...` environment overrides.

## Dependency and generated-artifact rules

- Add dependencies only when the code uses them.
- Keep `Cargo.lock` committed for this binary crate.
- Do not commit `target/` or other build artifacts.
- Regenerate and commit `cornucopia/` when SQL or schema changes.
- Preserve endpoint paths and config keys unless the task explicitly changes the public contract.
