import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { config } from "./config.js";
import { runSql } from "./duckdb.js";

export function createMcpServer(): McpServer {
  const server = new McpServer({
    name: "mizuumi-mcp",
    version: "0.1.0",
  });

  server.registerTool(
    "run_sql",
    {
      title: "Run SQL",
      description:
        "Execute a SQL query against the Mizuumi DuckDB Quack server. " +
        `Default catalog is "${config.defaultCatalog}", default schema is "${config.defaultSchema}". ` +
        "Example: SELECT * FROM unity.demo.orders",
      inputSchema: {
        sql: z.string().min(1).describe("SQL to execute via DuckDB Quack."),
      },
    },
    async ({ sql }) => {
      const result = await runSql(sql);
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
      };
    },
  );

  server.registerTool(
    "get_demo_orders",
    {
      title: "Get demo orders",
      description: "Return every row from unity.demo.orders.",
      inputSchema: {},
    },
    async () => {
      const result = await runSql(
        `SELECT * FROM ${config.defaultCatalog}.${config.defaultSchema}.orders`,
      );
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
      };
    },
  );

  return server;
}
