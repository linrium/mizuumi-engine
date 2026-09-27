# Managed Unity Catalog table example

This example creates one Delta table, `unity.demo.orders`, using `CREATE TABLE`
without a `LOCATION` clause. Unity Catalog allocates its managed storage path.
The application overwrites three sample rows on each run, reads them back, and
checks that Unity Catalog reports `table_type: MANAGED`.

From the repository root, after the local RustFS, Keycloak, and Unity Catalog
services are set up, deploy with:

```bash
./scripts/setup_spark_managed.sh
```

The script initializes the `demo` schema and its grants, mounts `main.py` from a
ConfigMap, builds the Spark image, installs the Spark Operator, and submits
`unity-catalog-managed`. It accepts the
same environment overrides and extra Helm options as `setup_spark.sh`. For an
image already available to the cluster:

```bash
SPARK_BUILD_IMAGE=false ./scripts/setup_spark_managed.sh
```

Watch the job and inspect its output with:

```bash
kubectl -n spark get sparkapplication unity-catalog-managed -w
kubectl -n spark logs unity-catalog-managed-driver --tail=-1
```

The final log prints the allocated storage location and three rows. Re-running
the setup script resubmits the application, keeps the same table, and replaces
its data.
