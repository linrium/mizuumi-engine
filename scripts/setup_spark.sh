#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
operator_namespace="${SPARK_OPERATOR_NAMESPACE:-spark-operator}"
job_namespace="${SPARK_NAMESPACE:-spark}"
operator_release="${SPARK_OPERATOR_RELEASE:-spark-operator}"
application_release="${SPARK_RELEASE:-spark}"
operator_version="${SPARK_OPERATOR_VERSION:-2.5.2}"
image_repository="${SPARK_IMAGE_REPOSITORY:-mizuumi/spark}"
image_tag="${SPARK_IMAGE_TAG:-4.0.1}"
spark_version="${SPARK_VERSION:-4.0.1}"
build_image="${SPARK_BUILD_IMAGE:-true}"
uc_namespace="${UNITYCATALOG_NAMESPACE:-tower}"
uc_release="${UNITYCATALOG_RELEASE:-unitycatalog}"
credentials_secret="${SPARK_CREDENTIALS_SECRET:-spark-runtime-credentials}"
trust_secret="${SPARK_TRUST_SECRET:-spark-rustfs-trust}"
work_dir=""

cleanup() {
  [[ -z "$work_dir" ]] || rm -rf "$work_dir"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage: ./scripts/setup_spark.sh [application Helm options]

Builds the Spark image, installs Kubeflow Spark Operator, and submits the
minimal medallion SparkApplication. Extra arguments are passed to the
application chart's `helm upgrade --install` command.

Environment overrides:
  SPARK_IMAGE_REPOSITORY    Image repository (default: mizuumi/spark)
  SPARK_IMAGE_TAG           Image tag (default: 4.0.1)
  SPARK_VERSION             Apache Spark base version (default: 4.0.1)
  SPARK_BUILD_IMAGE         Build/import the image (default: true)
  SPARK_NAMESPACE           Spark job namespace (default: spark)
  SPARK_OPERATOR_NAMESPACE  Operator namespace (default: spark-operator)
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for command in kubectl helm java keytool; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done
if [[ "$build_image" == true ]]; then
  command -v docker >/dev/null || { echo "Missing required command: docker" >&2; exit 1; }
fi

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Spark values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

uc_deployment="$(kubectl -n "$uc_namespace" get deployment \
  -l "app.kubernetes.io/instance=$uc_release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)"
if [[ -z "$uc_deployment" ]]; then
  echo "Unity Catalog is not installed. Run ./scripts/setup_unitycatalog.sh first." >&2
  exit 1
fi
"$repo_root/scripts/init_spark_catalog.sh"
kubectl -n "$uc_namespace" get secret unitycatalog-credentials >/dev/null 2>&1 || {
  echo "Missing Unity Catalog runtime credentials." >&2
  exit 1
}
[[ -f "$repo_root/k8s/storage/tls/ca.crt" ]] || {
  echo "Missing RustFS CA. Run ./scripts/setup_storage.sh first." >&2
  exit 1
}

if [[ "$build_image" == true ]]; then
  image="$image_repository:$image_tag"
  docker build --build-arg "SPARK_VERSION=$spark_version" -t "$image" "$repo_root/packages/spark"
  case "$context" in
    kind-*) kind load docker-image "$image" --name "${context#kind-}" ;;
    k3d-*) k3d image import "$image" --cluster "${context#k3d-}" ;;
    minikube|minikube-*) minikube image load "$image" ;;
    docker-desktop|rancher-desktop|orbstack|colima) ;;
    *)
      echo "Image built, but $context requires a registry or manual image import." >&2
      echo "Set SPARK_BUILD_IMAGE=false after making $image available to the cluster." >&2
      exit 1
      ;;
  esac
fi

kubectl create namespace "$job_namespace" --dry-run=client -o yaml | kubectl apply -f - >/dev/null

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/spark-setup.XXXXXX")"
java_home="$(java -XshowSettings:properties -version 2>&1 |
  sed -n 's/^[[:space:]]*java\.home = //p' | head -n 1)"
[[ -n "$java_home" && -f "$java_home/lib/security/cacerts" ]] || {
  echo "Cannot locate the active JRE truststore." >&2
  exit 1
}
cp "$java_home/lib/security/cacerts" "$work_dir/cacerts"
keytool -importcert -noprompt -storepass changeit -keystore "$work_dir/cacerts" \
  -alias mizuumi-rustfs -file "$repo_root/k8s/storage/tls/ca.crt" >/dev/null

uc_token="$(kubectl -n "$uc_namespace" exec deployment/"$uc_deployment" -c server -- \
  /bin/bash -ec 'cat /home/unitycatalog/etc/conf/token.txt')"
s3_access_key="$(kubectl -n "$uc_namespace" get secret unitycatalog-credentials \
  -o jsonpath='{.data.S3_ACCESS_KEY}' | base64 --decode)"
s3_secret_key="$(kubectl -n "$uc_namespace" get secret unitycatalog-credentials \
  -o jsonpath='{.data.S3_SECRET_KEY}' | base64 --decode)"

credentials_file="$work_dir/credentials.env"
umask 077
printf 'UNITY_CATALOG_TOKEN=%s\n' "$uc_token" > "$credentials_file"
printf 'UNITY_CATALOG_NAME=unity\n' >> "$credentials_file"
printf 'AWS_ACCESS_KEY_ID=%s\n' "$s3_access_key" >> "$credentials_file"
printf 'AWS_SECRET_ACCESS_KEY=%s\n' "$s3_secret_key" >> "$credentials_file"
printf 'AWS_REGION=us-east-1\n' >> "$credentials_file"
kubectl -n "$job_namespace" create secret generic "$credentials_secret" \
  --from-env-file="$credentials_file" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null
unset uc_token s3_access_key s3_secret_key

kubectl -n "$job_namespace" create secret generic "$trust_secret" \
  --from-file=cacerts="$work_dir/cacerts" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm repo add spark-operator https://kubeflow.github.io/spark-operator --force-update
helm repo update spark-operator
helm upgrade --install "$operator_release" spark-operator/spark-operator \
  --version "$operator_version" \
  --namespace "$operator_namespace" --create-namespace \
  --values "$repo_root/k8s/spark/operator-values.yaml" \
  --set "spark.jobNamespaces[0]=$job_namespace" \
  --wait --timeout 10m

helm upgrade --install "$application_release" "$repo_root/k8s/spark" \
  --namespace "$job_namespace" \
  --set-string "image.repository=$image_repository" \
  --set-string "image.tag=$image_tag" \
  --set-string "credentialsSecretName=$credentials_secret" \
  --set-string "trustSecretName=$trust_secret" \
  "$@"

echo "Spark Operator and medallion example are installed."
echo "Watch: kubectl -n $job_namespace get sparkapplications -w"
echo "Logs:  kubectl -n $job_namespace logs -l spark-role=driver --tail=-1"
