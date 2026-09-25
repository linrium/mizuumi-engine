# Spark Operator and application chart

This directory contains values for Kubeflow Spark Operator `2.5.2` and a
repository-owned Helm chart that submits the example `SparkApplication`.
Spark `4.0.1`, Delta Lake `4.3.1`, and the Unity Catalog Spark connector
`0.5.0` use the existing `unity` catalog backed by the RustFS
`s3://unitycatalog` bucket. The operator webhook is enabled because it injects
the Secret-backed environment into driver and executor pods.

The setup runs `scripts/init_spark_catalog.sh` to create the Bronze, Silver,
and Gold schemas through the Unity Catalog CLI and grant access to the RustFS
external location. Spark receives a short-lived RustFS STS session from the
setup script instead of mounting the underlying long-lived S3 identity.

Unity Catalog-to-RustFS automatic credential vending is not enabled here:
RustFS `1.0.0` currently rejects the request emitted by the Unity Catalog
`0.6.0` Java STS client, although its STS endpoint works with standard CLI
clients. Re-run setup to rotate the Spark session before its one-hour expiry.

Install Unity Catalog first, then run:

```sh
export UNITYCATALOG_USER_TOKEN='<access_token from bin/uc auth login>'
./scripts/setup_spark.sh
```

Spark setup is intentionally separate from `scripts/bootstrap.sh` because the
user-token login requires an interactive browser.

The script builds `packages/spark/Dockerfile`, imports the image into common
local Kubernetes runtimes, installs the operator, synchronizes runtime
credentials and RustFS trust, and installs this chart in the `spark`
namespace. Override the image for a registry-backed cluster with:

```sh
SPARK_IMAGE_REPOSITORY=registry.example.com/mizuumi/spark \
SPARK_IMAGE_TAG=4.0.1 ./scripts/setup_spark.sh
```

Set `SPARK_BUILD_IMAGE=false` when that image is already available. Additional
arguments are passed to this chart's `helm upgrade --install` command.

Generate the user token with the Unity Catalog CLI:

```sh
export UNITYCATALOG_USER_TOKEN="$(
  bin/uc auth login --output jsonPretty | jq -r .access_token
)"
```

You can alternatively put the raw token in a protected file and set
`UNITYCATALOG_USER_TOKEN_FILE`. Tokens expire; rerun login and setup before
submitting a job with an expired token.

Inspect a run with:

```sh
kubectl -n spark get sparkapplications
kubectl -n spark logs -l spark-role=driver --tail=-1
```

Delete and recreate the `medallion-minimal` resource to run the deterministic
example again. The pipeline overwrites its three Delta tables, so reruns do not
duplicate data.
