import { randomUUID } from "node:crypto";
import Fastify from "fastify";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import { isInitializeRequest } from "@modelcontextprotocol/sdk/types.js";
import { config } from "./config.js";
import { warmup } from "./duckdb.js";
import { createMcpServer } from "./mcp.js";

const app = Fastify({ logger: true });

const transports = new Map<string, StreamableHTTPServerTransport>();

app.get("/health", async () => ({
  status: "ok",
  service: "mizuumi-mcp",
  time: new Date().toISOString(),
}));

app.all("/mcp", async (request, reply) => {
  const sessionHeader = request.headers["mcp-session-id"];
  const sessionId = Array.isArray(sessionHeader) ? sessionHeader[0] : sessionHeader;

  let transport = sessionId ? transports.get(sessionId) : undefined;

  if (!transport) {
    const body = request.body as unknown;
    if (request.method !== "POST" || !isInitializeRequest(body)) {
      reply.code(400);
      return {
        jsonrpc: "2.0",
        error: { code: -32000, message: "Missing or invalid MCP session" },
        id: null,
      };
    }
    transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: () => randomUUID(),
      onsessioninitialized: (id) => {
        transports.set(id, transport!);
      },
    });
    transport.onclose = () => {
      if (transport?.sessionId) transports.delete(transport.sessionId);
    };
    const mcp = createMcpServer();
    await mcp.connect(transport);
  }

  await transport.handleRequest(request.raw, reply.raw, request.body);
  reply.hijack();
});

async function main() {
  await warmup().catch((error) => {
    app.log.warn({ err: error }, "DuckDB warmup failed; continuing");
  });
  await app.listen({ host: config.host, port: config.port });
  app.log.info(
    `Mizuumi MCP server listening on http://${config.host}:${config.port} (quack=${config.quackUri})`,
  );
}

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, async () => {
    app.log.info(`[${signal}] shutting down`);
    await app.close();
    process.exit(0);
  });
}

main().catch((error) => {
  app.log.error(error);
  process.exit(1);
});
