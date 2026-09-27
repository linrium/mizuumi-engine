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
  |-- write later staged Delta commit file ----------------------------->|
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

   The response's `table_id` should equal `TABLE_ID`. When `data_source_format` is omitted for a managed table, the Rust server defaults it to `DELTA`.

4. Read the registered metadata with `GET /tables/<catalog>.<schema>.<table>`. A non-admin caller needs the parent catalog/schema grants and a table grant such as `SELECT`; creating the table does not currently grant those automatically.

## What are Delta commits, and do I need them?

A Delta table is more than a set of data files. Its `_delta_log` records each change as a numbered **version**: which data files were added or removed, plus any schema or table-property changes. A reader uses that log to determine the table's current contents. A *Delta commit* is one such version of the log, not a new Unity Catalog table.

The managed-table creation calls above register the table's identity and location. They do not commit each later change to its data. The experimental `/delta/preview/commits` API is for a client that uses Unity Catalog as its **Delta commit coordinator**: it tells the server about a new log version so the server can enforce sequential version numbers and reject a conflicting version. The server keeps the commit's file name and metadata in PostgreSQL so clients can discover commits that have not yet been published to the regular Delta log.

For example:

```text
Version 0: client creates the initial Delta log at the staged table location
           and finalizes the MANAGED table through POST /tables.
Version 1: client writes a new, uniquely named commit file to object storage;
           POST /delta/preview/commits records its name and version in UC.
Backfill:  client publishes that commit into the regular numbered Delta log;
           POST /delta/preview/commits marks version 1 as backfilled.
```

Here, **backfill** means copying or publishing a coordinator-tracked commit into the ordinary Delta log, then telling UC it no longer needs to list that commit as pending. `GET /delta/preview/commits` returns those pending (unbackfilled) commits. Neither POST nor GET uploads or publishes files: the client does the object-storage work. This Rust server also does not verify that the referenced file exists.

You do **not** need this endpoint merely to create a managed table, and it does not apply to `EXTERNAL`, `VIEW`, or other non-managed/non-Delta table types. Use it only if your Delta writer is integrated with this coordinator protocol; a client that manages its own Delta log writes does not gain anything by posting commit metadata here. Do not treat a successful POST as proof that data files or a readable Delta log were created.

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
