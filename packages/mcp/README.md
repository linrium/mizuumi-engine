# Mizuumi MCP Server

A tiny Model Context Protocol server (Node.js + TypeScript + Fastify) that
exposes the Mizuumi DuckDB Quack endpoint as MCP tools.

## Tools

- `run_sql(sql)` — execute arbitrary SQL against DuckDB Quack.
- `get_demo_orders()` — shortcut for `SELECT * FROM unity.demo.orders`.

Queries are proxied through DuckDB's `quack_query(...)` to the deployed
`duck-server` (default URI `quack:query.mizuumi.test:443`) using
`@duckdb/node-api` in-process.

In-cluster the server is reachable at `https://mcp.mizuumi.test/mcp` via the
shared Caddy gateway.

## Local development

```bash
cd packages/mcp
npm install
export QUACK_TOKEN=$(kubectl -n duck-server get secret duck-server-credentials \
  -o jsonpath='{.data.QUACK_TOKEN}' | base64 --decode)
npm run dev
```

The MCP endpoint lives at `http://localhost:8080/mcp` (Streamable HTTP transport).
`GET /health` returns a liveness payload.

## In-cluster

See `k8s/mcp` and `scripts/mcp/` for the Helm chart and bootstrap/setup
scripts. They reuse the Quack token issued by `./scripts/duck-server/bootstrap.sh`.
