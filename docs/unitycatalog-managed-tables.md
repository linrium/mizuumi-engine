# Managed tables in the Rust Unity Catalog server

A managed table has a storage location allocated by Unity Catalog. Create it in two API calls: allocate a staging table, then finalize it as a `MANAGED` table using the returned location. The staging ID becomes the permanent `table_id`.

The Rust server stores table and commit metadata in PostgreSQL. The client writes Delta log and data files to object storage; these API calls do not write those files.

## Prerequisites

- A catalog and schema must exist. Either the schema or its parent catalog must have a managed `storage_root` configured. The server uses the schema's generated `storage_location` when present, otherwise the catalog's.
- Use a Unity Catalog bearer token with the writer or admin role for the POST requests. A non-admin token also needs the relevant table grants to read the finalized table or use the Delta commit endpoints.
- To request temporary storage credentials, configure an external location covering the allocated staging URL and the server's storage credential/STS integration. The account that created the staging table can request credentials for its unfinalized staging ID.

For example, a catalog can be configured with `storage_root: "s3://unitycatalog/managed-demo"`. Unity Catalog creates its managed storage location beneath that root, then allocates a table path like `.../__unitystorage/catalogs/<catalog-id>/tables/<table-id>`.

If starting with a new namespace, create the catalog with a storage root, then create a schema inside it:

```text
POST /catalogs  {"name":"demo","storage_root":"s3://unitycatalog/managed-demo"}
POST /schemas   {"catalog_name":"demo","name":"analytics"}
```

## Creation flow

```text
Client                                      Rust UC / PostgreSQL          Object storage
  |                                                |                           |
  |-- POST /staging-tables ----------------------->|                           |
  |<-- id + staging_location ----------------------| (uc_staging_tables)       |
  |                                                |                           |
  |-- POST /temporary-table-credentials --------->|  optional                 |
  |<-- short-lived credentials --------------------|                           |
  |                                                |                           |
  |-- write initial Delta log + data ------------------------------------->|
  |                                                |                           |
  |-- POST /tables (MANAGED, staging_location) --->|                           |
  |    validate stage; insert table with same ID   |                           |
  |    and finalize stage in one DB transaction    | (uc_tables)               |
  |<-- TableInfo -----------------------------------|                           |
  |                                                |                           |
  |   Optional later catalog-managed Delta write: |                           |
  |-- write staged Delta commit file ----------------------------------->|
  |-- POST /delta/preview/commits ---------------->| (uc_delta_commits)        |
  |-- publish/backfill that file --------------------------------------->|
  |-- POST /delta/preview/commits ---------------->| mark version backfilled   |
```

Finalization registers the name atomically with the staging update. The server checks that the catalog, schema, table name, staging location, and creating principal match, and rejects a staging location that has already been finalized.

## Create a table with the API

Set the API URL, CA certificate, bearer token, and names. This example assumes the catalog and schema already exist and have managed storage configured.

```bash
export UC_API='https://unitycatalog.mizuumi.test/api/2.1/unity-catalog'
export UC_CA='k8s/auth/tls/ca.crt'
export UC_TOKEN='<bearer-token>'
export UC_CATALOG='unity'
export UC_SCHEMA='gold'
export UC_TABLE='managed_orders'
```

1. Allocate the staging table and keep both returned values:

   ```bash
   STAGE="$(jq -nc \
     --arg catalog "$UC_CATALOG" --arg schema "$UC_SCHEMA" --arg name "$UC_TABLE" \
     '{catalog_name:$catalog,schema_name:$schema,name:$name}' |
     curl -fsS --cacert "$UC_CA" -H "Authorization: Bearer $UC_TOKEN" \
       -H 'Content-Type: application/json' --data-binary @- "$UC_API/staging-tables")"
   TABLE_ID="$(jq -r '.id' <<<"$STAGE")"
   LOCATION="$(jq -r '.staging_location' <<<"$STAGE")"
   ```

2. Initialize the table data at `LOCATION`. For a Delta table, the client writes its initial version 0 log and any data files there. If it needs temporary credentials, POST this body to `/temporary-table-credentials` using the same bearer token:

   ```json
   {"table_id":"<TABLE_ID>","operation":"READ_WRITE"}
   ```

   The response contains short-lived storage credentials. Keep them private. The Rust server checks for a matching external location before vending them.

3. Finalize the table. Use the same catalog, schema, and table name as the staging request, and pass the exact `staging_location` as `storage_location`:

   ```bash
   jq -nc \
     --arg catalog "$UC_CATALOG" --arg schema "$UC_SCHEMA" --arg name "$UC_TABLE" \
     --arg location "$LOCATION" \
     '{catalog_name:$catalog,schema_name:$schema,name:$name,
       table_type:"MANAGED",data_source_format:"DELTA",storage_location:$location,
       columns:[{name:"id",type_name:"LONG",position:0}]}' |
     curl -fsS --cacert "$UC_CA" -H "Authorization: Bearer $UC_TOKEN" \
       -H 'Content-Type: application/json' --data-binary @- "$UC_API/tables" | jq .
   ```

   The response's `table_id` should equal `TABLE_ID`. When `data_source_format` is omitted for a managed table, the Rust server defaults it to `DELTA`. This API example registers a UC-managed table; it does not by itself create a Delta `catalogManaged`-enabled log or prove compatibility with a coordinator-aware engine.

4. Read the registered metadata with `GET /tables/<catalog>.<schema>.<table>`. A non-admin caller needs the parent catalog/schema grants and a table grant such as `SELECT`; creating the table does not currently grant those automatically.

## What are Delta commits, and do I need them?

A Delta table is more than a set of Parquet data files. Its `_delta_log` records each change as a numbered **version**: which data files were added or removed, plus any schema or table-property changes. A reader uses that log to determine the table's contents. A *Delta commit* is one such version of the log, not a new Unity Catalog table.

There are two different meanings of *managed* here:

- Unity Catalog `table_type: "MANAGED"` means UC allocates the table's storage location. That is what the staging and table-creation calls above establish.
- Delta's `catalogManaged` table feature means a compatible writer and reader use a catalog to coordinate and discover commits. Setting the UC table type alone does **not** enable that Delta feature or cause an engine to call `/delta/preview/commits`.

The experimental `/delta/preview/commits` API is for clients using UC as a **commit coordinator**. That means a writer stages a new Delta log file, asks UC to accept it as the next table-wide version, and later publishes it into the ordinary numbered `_delta_log`. UC stores the accepted file name and version in PostgreSQL; a coordinator-aware reader can query UC for commits not yet published to the ordinary log. Clients that use the regular filesystem-based Delta protocol instead commit directly to `_delta_log` and do not use this endpoint. See [Delta's catalog-managed table overview](https://docs.delta.io/delta-catalog-managed-tables/) for the protocol distinction.

### Concrete example: two writers and one insert

Suppose `orders` has version 0 at `s3://warehouse/tables/orders-123`. Writer A inserts order 101:

```text
1. A writes a Parquet data file, e.g. part-A.parquet.
2. A writes a uniquely named staged Delta commit file for version 1. Its
   contents include an "add part-A.parquet" action.
3. A calls POST /delta/preview/commits with version 1 and that file name.
   UC accepts version 1; the ordinary _delta_log/00000000000000000001.json
   may not exist yet.
4. A publishes the staged commit as that ordinary numbered log file.
5. Only after publication succeeds, A reports latest_backfilled_version: 1
   to UC. The Parquet data file is not copied during this step.
```

Version numbers belong to the **table**, not to each writer. If writer B also started from version 0 and submits a *different* version 1 after A's commit was accepted, UC rejects B's attempt. B must read A's commit, check for conflicts, and retry its own change as version 2. UC permits an idempotent retry of A's already-accepted version and file name.

**Backfill** here means publishing an accepted staged *log file* into the ordinary numbered `_delta_log`, then telling UC it is published. It is not a bulk reload or copy of the Parquet data. Until then, coordinator-aware readers can discover the pending commit through `GET /delta/preview/commits`, while readers using only the ordinary log might not see it. Publishing also clears UC's pending list; this Rust server limits a table to 10 unbackfilled commits. Never mark a version backfilled before confirming publication.

The Rust server records and orders commits; it does **not** upload, publish, or verify the staged log or data files. A successful API response alone does not prove that a readable Delta table exists. You do **not** need this endpoint merely to create a UC-managed table. Its current implementation accepts only a `MANAGED` Delta table, not `EXTERNAL`, `VIEW`, or other table types.

### Which engines use this flow?

An engine is a *writer* when it writes Delta data, but that does not mean it uses UC-coordinated commits:

| Engine | In this project or current documentation |
| --- | --- |
| Spark | The repo's [minimal example](../packages/spark/examples/minimal/main.py) uses `USING DELTA LOCATION` and `INSERT OVERWRITE`: it writes external Delta tables and does not call our staging or commit endpoints. Delta Lake supports catalog-managed commits when a compatible catalog, engine, and table feature are configured ([Delta docs](https://docs.delta.io/delta-catalog-managed-tables/)). |
| Daft | `write_deltalake` can target a Unity Catalog table, but its published API does not establish that it uses this UC commit coordinator; do not assume it calls our endpoint ([Daft API](https://docs.daft.ai/en/stable/api/dataframe/)). |
| DuckDB | Its Unity Catalog extension supports `INSERT` and says it uses catalog-managed commits when a table requires them, but does not currently support `CREATE TABLE` there ([DuckDB docs](https://duckdb.org/docs/current/core_extensions/unity_catalog)). Compatibility with this Rust server has not been tested. |

If you need coordinated commits across engines, verify that the created Delta log enables `catalogManaged`, that each engine supports it, and that each engine works with this server's experimental endpoint. A UC `MANAGED` registration by itself is insufficient.

## Subsequent Delta commits

`GET` and `POST /delta/preview/commits` apply only to a managed Delta table. The initial table version is 0. For the next version, first write the staged commit file to object storage, then register version 1 with the coordinator:

```json
{
  "table_id": "<TABLE_ID>",
  "table_uri": "<LOCATION>",
  "commit_info": {
    "version": 1,
    "timestamp": 1720000000000,
    "file_name": "00000000000000000001.<uuid>.json",
    "file_size": 1234,
    "file_modification_timestamp": 1720000000000
  }
}
```

Send a JSON body to `GET /delta/preview/commits` with `table_id`, `table_uri`, and `start_version` (optionally `end_version`) to list unbackfilled commits. After publishing a commit in object storage, POST `{"table_id":"<TABLE_ID>","table_uri":"<LOCATION>","latest_backfilled_version":1}` to mark it backfilled. The server permits up to 10 unbackfilled commits per table and accepts an idempotent replay of the same version and file name.

The server tracks commit metadata; it does not upload, publish, or verify the Delta files. Clients must keep the object-store log consistent with the versions they register.

The current Rust implementation also does not remove object-store files when a managed table is deleted; deletion removes the catalog metadata.

## Where the state lives and how to test

PostgreSQL stores staging allocations in `uc_staging_tables`, finalized table metadata in `uc_tables`, and coordinator state in `uc_delta_table_state` and `uc_delta_commits`. The actual table data belongs at the allocated object-storage URL.

Run [`scripts/test_unitycatalog_table_types.sh`](../scripts/test_unitycatalog_table_types.sh) with `UNITYCATALOG_TEST_STORAGE_ROOT=s3://<bucket>/<test-prefix>` to smoke-test staging, all table types, and Delta commits. That script creates a temporary catalog and **deletes it on exit**, so its test tables will not remain visible afterward. It does not write object-store data.

The request and response shapes are also described in [`packages/unitycatalog/openapi.yaml`](../packages/unitycatalog/openapi.yaml).
