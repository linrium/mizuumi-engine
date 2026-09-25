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
credential_ttl="${SPARK_CREDENTIAL_TTL:-3600}"
work_dir=""
rustfs_pid=""

cleanup() {
  [[ -z "$rustfs_pid" ]] || { kill "$rustfs_pid" 2>/dev/null || true; wait "$rustfs_pid" 2>/dev/null || true; }
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
  UNITYCATALOG_USER_TOKEN   Token returned by `bin/uc auth login`
  UNITYCATALOG_USER_TOKEN_FILE  File containing that user token
  SPARK_CREDENTIAL_TTL      RustFS STS lifetime in seconds (default: 3600)
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for command in kubectl helm java keytool curl python3; do
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
if [[ -n "${UNITYCATALOG_USER_TOKEN:-}" ]]; then
  uc_token="$UNITYCATALOG_USER_TOKEN"
elif [[ -n "${UNITYCATALOG_USER_TOKEN_FILE:-}" ]]; then
  [[ -f "$UNITYCATALOG_USER_TOKEN_FILE" ]] || {
    echo "Unity Catalog token file does not exist: $UNITYCATALOG_USER_TOKEN_FILE" >&2
    exit 1
  }
  uc_token="$(<"$UNITYCATALOG_USER_TOKEN_FILE")"
else
  echo "Set UNITYCATALOG_USER_TOKEN or UNITYCATALOG_USER_TOKEN_FILE." >&2
  echo "Obtain a user token with: bin/uc auth login --output jsonPretty" >&2
  exit 1
fi
[[ -n "$uc_token" ]] || { echo "Unity Catalog user token is empty." >&2; exit 1; }
[[ "$uc_token" != *$'\n'* && "$uc_token" != *$'\r'* ]] || {
  echo "Unity Catalog user token must be a single line." >&2
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

credentials_file="$work_dir/credentials.env"
umask 077
printf 'UNITY_CATALOG_TOKEN=%s\n' "$uc_token" > "$credentials_file"
printf 'UNITY_CATALOG_NAME=unity\n' >> "$credentials_file"
unset uc_token

kubectl -n "$uc_namespace" get secret unitycatalog-credentials >/dev/null 2>&1 || {
  echo "Missing Unity Catalog RustFS credentials." >&2
  exit 1
}
s3_access_key="$(kubectl -n "$uc_namespace" get secret unitycatalog-credentials \
  -o jsonpath='{.data.S3_ACCESS_KEY}' | base64 --decode)"
s3_secret_key="$(kubectl -n "$uc_namespace" get secret unitycatalog-credentials \
  -o jsonpath='{.data.S3_SECRET_KEY}' | base64 --decode)"
kubectl -n storage port-forward --address 127.0.0.1 service/rustfs-svc 19000:9000 \
  >/dev/null 2>&1 &
rustfs_pid=$!
for attempt in {1..60}; do
  if curl -fsS --cacert "$repo_root/k8s/storage/tls/ca.crt" \
    https://127.0.0.1:19000/minio/health/live >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "RustFS is unavailable." >&2; exit 1; fi
  sleep 1
done
sts_file="$work_dir/sts.xml"
sts_policy='{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":["s3:GetBucketLocation","s3:ListBucket","s3:ListBucketMultipartUploads"],"Resource":["arn:aws:s3:::unitycatalog"]},{"Effect":"Allow","Action":["s3:GetObject","s3:PutObject","s3:DeleteObject","s3:AbortMultipartUpload","s3:ListMultipartUploadParts"],"Resource":["arn:aws:s3:::unitycatalog/*"]}]}'
sts_status="$(curl -sS --cacert "$repo_root/k8s/storage/tls/ca.crt" \
  -o "$sts_file" -w '%{http_code}' -X POST \
  --aws-sigv4 "aws:amz:us-east-1:sts" --user "$s3_access_key:$s3_secret_key" \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  --data-urlencode 'Action=AssumeRole' --data-urlencode 'Version=2011-06-15' \
  --data-urlencode 'RoleArn=arn:aws:iam::000000000000:role/unitycatalog' \
  --data-urlencode 'RoleSessionName=mizuumi-spark' \
  --data-urlencode "Policy=$sts_policy" \
  --data-urlencode "DurationSeconds=$credential_ttl" https://127.0.0.1:19000/)"
unset s3_access_key s3_secret_key sts_policy
[[ "$sts_status" == 200 ]] || {
  echo "RustFS temporary credential request failed (HTTP $sts_status)." >&2
  exit 1
}
python3 - "$sts_file" "$credentials_file" <<'PY'
import sys
import xml.etree.ElementTree as ET

root = ET.parse(sys.argv[1]).getroot()
values = {}
for element in root.iter():
    name = element.tag.rsplit("}", 1)[-1]
    if name in {"AccessKeyId", "SecretAccessKey", "SessionToken"}:
        values[name] = element.text
required = {"AccessKeyId", "SecretAccessKey", "SessionToken"}
if values.keys() != required or not all(values.values()):
    raise SystemExit("RustFS returned incomplete temporary credentials")
with open(sys.argv[2], "a", encoding="utf-8") as output:
    output.write(f"AWS_ACCESS_KEY_ID={values['AccessKeyId']}\n")
    output.write(f"AWS_SECRET_ACCESS_KEY={values['SecretAccessKey']}\n")
    output.write(f"AWS_SESSION_TOKEN={values['SessionToken']}\n")
    output.write("AWS_REGION=us-east-1\n")
PY
kubectl -n "$job_namespace" create secret generic "$credentials_secret" \
  --from-env-file="$credentials_file" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

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
