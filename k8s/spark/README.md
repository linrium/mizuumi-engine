# Spark Operator and application chart

This directory contains values for Kubeflow Spark Operator `2.5.2` and a
repository-owned Helm chart that submits the example `SparkApplication`.
Spark `4.0.1`, Delta Lake `4.0.0`, and the Unity Catalog Spark connector
`0.3.0` use the existing `unity` catalog backed by the RustFS
`s3://unitycatalog` bucket. The operator webhook is enabled because it injects
the Secret-backed environment into driver and executor pods.

The setup runs `scripts/init_spark_catalog.sh` to create the Bronze, Silver,
and Gold schemas through the Unity Catalog CLI. Data uses matching Delta paths
in RustFS with the dedicated S3 credentials; this avoids the current
Unity Catalog OSS/RustFS STS incompatibility while retaining catalog/schema
discovery through the Spark connector.

Install Unity Catalog first, then run:

```sh
./scripts/setup_spark.sh
```

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

Inspect a run with:

```sh
kubectl -n spark get sparkapplications
kubectl -n spark logs -l spark-role=driver --tail=-1
```

Delete and recreate the `medallion-minimal` resource to run the deterministic
example again. The pipeline overwrites its three Delta tables, so reruns do not
duplicate data.
