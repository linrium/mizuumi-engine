# Repository Guidelines

## Project Structure & Module Organization

This repository assembles the Mizuumi platform. Kubernetes and Helm definitions live in `k8s/<service>/`; operational setup, bootstrap, and validation scripts live in `scripts/<service>/`. Application code is under `packages/`: `studio/` is a Next.js UI, `unitycatalog/server/` is the Rust API, `simulators/temenos/` is a TypeScript service, and `spark/` and `redpanda/` contain Python runtimes and examples. Keep this layout: extend services in place and mirror new services across `packages/`, `k8s/`, and `scripts/`. Treat `packages/unitycatalog/unitycatalog/` as upstream code. Read nested `AGENTS.md` files before changing Studio or the Rust server.

## Build, Test, and Development Commands

- `./scripts/bootstrap.sh` provisions the complete local stack; use `--no-forward` to skip gateway forwarding.
- `./scripts/forward.sh` exposes and verifies local service endpoints.
- In `packages/studio/`, run `npm install`, `npm run dev`, `npm run build`, and `npm run lint` for local development, production validation, and Biome checks.
- In `packages/unitycatalog/server/`, run `cargo fmt`, `cargo check`, `cargo test`, and `cargo run`.
- In `packages/simulators/temenos/`, use `npm run dev`, `npm run build`, and the `npm run db:*` commands for schema generation, migrations, seeding, or reset.

Run service-specific scripts from the repository root unless their documentation says otherwise. Most integration workflows require Docker, Kubernetes, Helm, `kubectl`, and locally generated TLS credentials.

## Coding Style & Naming Conventions

Use two-space indentation and Biome formatting for Studio TypeScript; omit semicolons as configured in `packages/studio/biome.json`. Follow `rustfmt` defaults and feature-first modules (`src/features/<feature>/`) in the Rust server. Shell scripts should use Bash, `set -euo pipefail`, quoted variables, and `snake_case` names. Name Kubernetes resources and directories with lowercase kebab-case. Never hand-edit generated Rust under `packages/unitycatalog/server/cornucopia/`; regenerate it from `queries/` and `schema.sql`.

## Testing Guidelines

Place Rust unit tests beside their modules with `#[cfg(test)]`; run `cargo test` before submission. For UI changes, run `npm run lint` and `npm run build`. Exercise infrastructure changes with the relevant `scripts/<service>/test-*.sh` or validation script and document any checks that require a live cluster. Add regression coverage near the changed component.

## Commit & Pull Request Guidelines

Recent history uses terse lowercase subjects such as `update`; improve on this by using a concise imperative subject that names the area, for example `studio: validate agent names`. Keep commits focused. Pull requests should explain the change and operational impact, list commands run, link the issue when applicable, and include screenshots for UI changes. Call out configuration, credential, schema, generated-file, or deployment changes explicitly; never commit secrets or local TLS material.
