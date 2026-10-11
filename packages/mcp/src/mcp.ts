import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { config } from "./config.js";
import { runSql, type QueryResult } from "./duckdb.js";

const DEFAULT_ROW_LIMIT = 500;
const MAX_ROW_LIMIT = 5000;

function identifier(name: string): string {
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) {
    throw new Error(
      `Invalid identifier: ${name}. Use letters, digits, and underscores only.`,
    );
  }
  return `"${name}"`;
}

function literal(value: string): string {
  return `'${value.replace(/'/g, "''")}'`;
}

function orDefault(value: string | undefined, fallback: string): string {
  const trimmed = (value ?? "").trim();
  return trimmed === "" ? fallback : trimmed;
}

function reply(result: QueryResult) {
  return {
    content: [
      {
        type: "text" as const,
        text: JSON.stringify(result, null, 2),
      },
    ],
  };
}

export function createMcpServer(): McpServer {
  const server = new McpServer({
    name: "mizuumi-mcp",
    version: "0.1.0",
  });

  server.registerTool(
    "list_catalogs",
    {
      title: "List catalogs",
      description:
        "Return every attached DuckDB catalog name (databases) available on the server.",
      inputSchema: {},
    },
    async () =>
      reply(
        await runSql(
          "SELECT database_name FROM duckdb_databases() ORDER BY database_name",
        ),
      ),
  );

  server.registerTool(
    "list_schemas",
    {
      title: "List schemas",
      description:
        `List schemas, optionally filtered by catalog (default "${config.defaultCatalog}"). ` +
        'Pass catalog = "*" to list schemas across all catalogs, or "" for the default.',
      inputSchema: {
        catalog: z
          .string()
          .describe(
            `Catalog filter. "" uses the default ("${config.defaultCatalog}"); "*" means all catalogs.`,
          ),
      },
    },
    async ({ catalog }) => {
      const target = orDefault(catalog, config.defaultCatalog);
      const where =
        target === "*" ? "" : `WHERE catalog_name = ${literal(target)}`;
      const sql =
        "SELECT catalog_name, schema_name FROM information_schema.schemata " +
        `${where} ORDER BY catalog_name, schema_name`;
      return reply(await runSql(sql));
    },
  );

  server.registerTool(
    "list_tables",
    {
      title: "List tables",
      description:
        "List tables and views, optionally filtered by catalog and/or schema. " +
        `Defaults to catalog "${config.defaultCatalog}", schema "${config.defaultSchema}". ` +
        'Pass "*" to disable a filter, or "" to use the default.',
      inputSchema: {
        catalog: z
          .string()
          .describe(
            `Catalog filter. "" uses "${config.defaultCatalog}"; "*" means all.`,
          ),
        schema: z
          .string()
          .describe(
            `Schema filter. "" uses "${config.defaultSchema}"; "*" means all.`,
          ),
      },
    },
    async ({ catalog, schema }) => {
      const catalogTarget = orDefault(catalog, config.defaultCatalog);
      const schemaTarget = orDefault(schema, config.defaultSchema);
      const filters: string[] = [];
      if (catalogTarget !== "*")
        filters.push(`table_catalog = ${literal(catalogTarget)}`);
      if (schemaTarget !== "*")
        filters.push(`table_schema = ${literal(schemaTarget)}`);
      const where = filters.length ? `WHERE ${filters.join(" AND ")}` : "";
      const sql =
        "SELECT table_catalog, table_schema, table_name, table_type " +
        "FROM information_schema.tables " +
        `${where} ORDER BY table_catalog, table_schema, table_name`;
      return reply(await runSql(sql));
    },
  );

  server.registerTool(
    "describe_table",
    {
      title: "Describe table",
      description:
        "Return the columns, types, and nullability for a specific table. " +
        `Catalog and schema default to "${config.defaultCatalog}" and "${config.defaultSchema}" when "" is passed.`,
      inputSchema: {
        table: z.string().min(1).describe("Table name."),
        schema: z
          .string()
          .describe(
            `Schema name. "" uses "${config.defaultSchema}".`,
          ),
        catalog: z
          .string()
          .describe(
            `Catalog name. "" uses "${config.defaultCatalog}".`,
          ),
      },
    },
    async ({ table, schema, catalog }) => {
      const catalogTarget = orDefault(catalog, config.defaultCatalog);
      const schemaTarget = orDefault(schema, config.defaultSchema);
      const sql =
        "SELECT column_name, data_type, is_nullable, ordinal_position " +
        "FROM information_schema.columns " +
        `WHERE table_catalog = ${literal(catalogTarget)} ` +
        `  AND table_schema = ${literal(schemaTarget)} ` +
        `  AND table_name = ${literal(table)} ` +
        "ORDER BY ordinal_position";
      return reply(await runSql(sql));
    },
  );

  server.registerTool(
    "preview_table",
    {
      title: "Preview table rows",
      description:
        "Return the first N rows of a table (safe, read-only). " +
        `Defaults to ${DEFAULT_ROW_LIMIT} rows; max ${MAX_ROW_LIMIT}. ` +
        'Pass "" for schema/catalog to use defaults; limit=0 uses the default.',
      inputSchema: {
        table: z.string().min(1).describe("Table name."),
        schema: z.string().describe(`Schema. "" uses "${config.defaultSchema}".`),
        catalog: z
          .string()
          .describe(`Catalog. "" uses "${config.defaultCatalog}".`),
        limit: z
          .number()
          .int()
          .min(0)
          .max(MAX_ROW_LIMIT)
          .describe(`Row limit; 0 uses default ${DEFAULT_ROW_LIMIT}.`),
      },
    },
    async ({ table, schema, catalog, limit }) => {
      const catalogTarget = orDefault(catalog, config.defaultCatalog);
      const schemaTarget = orDefault(schema, config.defaultSchema);
      const qualified = [
        identifier(catalogTarget),
        identifier(schemaTarget),
        identifier(table),
      ].join(".");
      const rowLimit = Math.min(
        limit && limit > 0 ? limit : DEFAULT_ROW_LIMIT,
        MAX_ROW_LIMIT,
      );
      return reply(await runSql(`SELECT * FROM ${qualified} LIMIT ${rowLimit}`));
    },
  );

  server.registerTool(
    "run_sql",
    {
      title: "Run SQL",
      description:
        "Execute an arbitrary SQL query against the Mizuumi DuckDB Quack server. " +
        `Default catalog is "${config.defaultCatalog}", default schema is "${config.defaultSchema}", ` +
        "so prefer fully qualified names like unity.demo.orders. " +
        "Prefer discovery tools (list_catalogs, list_schemas, list_tables, describe_table) " +
        "when you don't yet know the schema.",
      inputSchema: {
        sql: z.string().min(1).describe("SQL to execute via DuckDB Quack."),
      },
    },
    async ({ sql }) => reply(await runSql(sql)),
  );

  return server;
}
