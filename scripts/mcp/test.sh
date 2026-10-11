#!/usr/bin/env bash
set -euo pipefail

namespace="${MCP_NAMESPACE:-mcp}"
release="${MCP_RELEASE:-mcp}"

command -v kubectl >/dev/null || { echo "Missing required command: kubectl" >&2; exit 1; }

kubectl -n "$namespace" wait --for=condition=available deployment/"$release" --timeout=120s

echo "Probing MCP health endpoint via deployment/$release"
kubectl -n "$namespace" exec deployment/"$release" -- node -e '
const http = require("http");
http.get("http://127.0.0.1:8080/health", (res) => {
  let body = "";
  res.on("data", (chunk) => { body += chunk; });
  res.on("end", () => {
    if (res.statusCode !== 200) { console.error("Unexpected status", res.statusCode, body); process.exit(1); }
    console.log("Health:", body);
  });
}).on("error", (error) => { console.error(error); process.exit(1); });
'

echo "Issuing MCP initialize + tools/call(get_demo_orders) via Streamable HTTP"
kubectl -n "$namespace" exec deployment/"$release" -- node -e '
const http = require("http");

function request(path, headers, body) {
  return new Promise((resolve, reject) => {
    const data = body ? JSON.stringify(body) : "";
    const req = http.request({
      host: "127.0.0.1",
      port: 8080,
      method: body ? "POST" : "GET",
      path,
      headers: {
        "content-type": "application/json",
        "accept": "application/json, text/event-stream",
        ...(data ? { "content-length": Buffer.byteLength(data) } : {}),
        ...headers,
      },
    }, (res) => {
      let buf = "";
      res.on("data", (chunk) => { buf += chunk; });
      res.on("end", () => resolve({ status: res.statusCode, headers: res.headers, body: buf }));
    });
    req.on("error", reject);
    if (data) req.write(data);
    req.end();
  });
}

function parse(body) {
  if (!body) return null;
  const trimmed = body.trim();
  if (trimmed.startsWith("{")) return JSON.parse(trimmed);
  for (const line of trimmed.split(/\r?\n/)) {
    if (line.startsWith("data:")) return JSON.parse(line.slice(5).trim());
  }
  throw new Error("Unparseable MCP response: " + body);
}

(async () => {
  const init = await request("/mcp", {}, {
    jsonrpc: "2.0", id: 1, method: "initialize",
    params: {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "mcp-test", version: "0.0.0" },
    },
  });
  const sessionId = init.headers["mcp-session-id"];
  if (!sessionId) { console.error("No session id. body=", init.body); process.exit(1); }
  console.log("Initialized session:", sessionId);

  await request("/mcp", { "mcp-session-id": sessionId }, {
    jsonrpc: "2.0", method: "notifications/initialized", params: {},
  });

  const call = await request("/mcp", { "mcp-session-id": sessionId }, {
    jsonrpc: "2.0", id: 2, method: "tools/call",
    params: { name: "get_demo_orders", arguments: {} },
  });
  const parsed = parse(call.body);
  if (parsed && parsed.error) {
    console.error("tools/call failed:", JSON.stringify(parsed.error));
    process.exit(1);
  }
  const text = parsed && parsed.result && parsed.result.content && parsed.result.content[0] && parsed.result.content[0].text;
  console.log("get_demo_orders result:");
  console.log(text ?? JSON.stringify(parsed, null, 2));
})().catch((error) => { console.error(error); process.exit(1); });
'
