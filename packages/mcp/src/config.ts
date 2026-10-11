function required(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`Missing required env var: ${name}`);
  return value;
}

export const config = {
  port: Number(process.env.PORT ?? 8080),
  host: process.env.HOST ?? "0.0.0.0",
  quackUri: process.env.QUACK_URI ?? "quack:query.mizuumi.test:443",
  quackToken: required("QUACK_TOKEN"),
  gatewayCaFile:
    process.env.QUACK_CA_FILE ?? process.env.NODE_EXTRA_CA_CERTS ?? "",
  defaultCatalog: process.env.UNITY_CATALOG_NAME ?? "unity",
  defaultSchema: process.env.UNITY_CATALOG_DEFAULT_SCHEMA ?? "demo",
};
