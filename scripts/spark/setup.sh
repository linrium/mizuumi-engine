#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
operator_namespace="${SPARK_OPERATOR_NAMESPACE:-spark-operator}"
job_namespace="${SPARK_NAMESPACE:-spark}"
operator_release="${SPARK_OPERATOR_RELEASE:-spark-operator}"
application_release="${SPARK_RELEASE:-spark}"
operator_version="${SPARK_OPERATOR_VERSION:-2.5.2}"
image_repository="${SPARK_IMAGE_REPOSITORY:-mizuumi/spark}"
image_tag="${SPARK_IMAGE_TAG:-4.0.1-uc-kafka-rustfs-baked}"
spark_version="${SPARK_VERSION:-4.0.1}"
build_image="${SPARK_BUILD_IMAGE:-true}"
uc_namespace="${UNITYCATALOG_NAMESPACE:-tower}"
uc_release="${UNITYCATALOG_RELEASE:-unitycatalog}"
credentials_secret="${SPARK_CREDENTIALS_SECRET:-spark-runtime-credentials}"
uc_credentials_secret="${UNITYCATALOG_CREDENTIALS_SECRET:-unitycatalog-credentials}"
trust_secret="${SPARK_TRUST_SECRET:-spark-rustfs-trust}"
catalog="${SPARK_UNITY_CATALOG:-unity}"
storage_root="${SPARK_UNITY_STORAGE_ROOT:-s3://unitycatalog/spark}"
application_script="${SPARK_APPLICATION_SCRIPT:-$repo_root/packages/spark/examples/minimal/main.py}"
application_source="${SPARK_APPLICATION_SOURCE:-rustfs}"
script_configmap="${SPARK_APPLICATION_SCRIPT_CONFIGMAP:-}"
script_mount_path="${SPARK_APPLICATION_SCRIPT_MOUNT_PATH:-}"
artifact_bucket="${SPARK_APPLICATION_ARTIFACT_BUCKET:-unitycatalog}"
artifact_prefix="${SPARK_APPLICATION_ARTIFACT_PREFIX:-spark-applications}"
artifact_url_secret="${SPARK_APPLICATION_URL_SECRET:-spark-application-artifact}"
artifact_url_expiration="${SPARK_APPLICATION_URL_EXPIRATION:-3600}"
rustfs_namespace="${RUSTFS_NAMESPACE:-storage}"
rustfs_service="${RUSTFS_SERVICE:-rustfs-svc}"
rustfs_upload_port="${SPARK_ARTIFACT_UPLOAD_PORT:-19001}"
work_dir=""
rustfs_pid=""

cleanup() {
  [[ -z "$rustfs_pid" ]] || { kill "$rustfs_pid" 2>/dev/null || true; wait "$rustfs_pid" 2>/dev/null || true; }
  [[ -z "$work_dir" ]] || rm -rf "$work_dir"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage: ./scripts/spark/setup.sh [application Helm options]

Builds the Spark image, installs Kubeflow Spark Operator, and submits the
minimal medallion SparkApplication. Extra arguments are passed to the
application chart's `helm upgrade --install` command.

Environment overrides:
  SPARK_IMAGE_REPOSITORY    Image repository (default: mizuumi/spark)
  SPARK_IMAGE_TAG           Image tag (default: 4.0.1-uc-kafka-rustfs-baked)
  SPARK_VERSION             Apache Spark base version (default: 4.0.1)
  SPARK_BUILD_IMAGE         Build/import the image (default: true)
  SPARK_APPLICATION_SOURCE  rustfs, configmap, or image (default: rustfs)
  SPARK_APPLICATION_SCRIPT  Local Python file uploaded for the application
  SPARK_APPLICATION_ARTIFACT_BUCKET  RustFS bucket (default: unitycatalog)
  SPARK_APPLICATION_ARTIFACT_PREFIX  Object prefix (default: spark-applications)
  SPARK_APPLICATION_URL_EXPIRATION  Presigned GET lifetime in seconds (default: 3600)
  SPARK_NAMESPACE           Spark job namespace (default: spark)
  SPARK_OPERATOR_NAMESPACE  Operator namespace (default: spark-operator)
  UNITYCATALOG_CREDENTIALS_SECRET  Source Secret containing the Keycloak
                              service principal (default: unitycatalog-credentials)
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
case "$application_source" in
  rustfs)
    for command in curl openssl python3; do
      command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
    done
    ;;
  configmap|image) ;;
  *) echo "SPARK_APPLICATION_SOURCE must be rustfs, configmap, or image." >&2; exit 1 ;;
esac
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
  echo "Unity Catalog is not installed. Run ./scripts/unitycatalog/setup.sh first." >&2
  exit 1
fi
kubectl -n "$uc_namespace" get secret "$uc_credentials_secret" >/dev/null 2>&1 || {
  echo "Missing $uc_namespace/$uc_credentials_secret; rerun Unity Catalog bootstrap and setup." >&2
  exit 1
}
keycloak_client_id="$(kubectl -n "$uc_namespace" get secret "$uc_credentials_secret" \
  -o jsonpath='{.data.KEYCLOAK_CLIENT_ID}' | base64 --decode)"
keycloak_client_secret="$(kubectl -n "$uc_namespace" get secret "$uc_credentials_secret" \
  -o jsonpath='{.data.KEYCLOAK_CLIENT_SECRET}' | base64 --decode)"
[[ -n "$keycloak_client_id" && -n "$keycloak_client_secret" ]] || {
  echo "Keycloak Spark service-principal credentials are missing; rerun ./scripts/unitycatalog/bootstrap.sh and ./scripts/unitycatalog/setup.sh." >&2
  exit 1
}
UNITYCATALOG_PRINCIPAL="$keycloak_client_id" "$repo_root/scripts/spark/init-catalog.sh"
[[ -f "$repo_root/k8s/storage/tls/ca.crt" ]] || {
  echo "Missing RustFS CA. Run ./scripts/storage/setup.sh first." >&2
  exit 1
}
[[ -f "$repo_root/k8s/auth/tls/ca.crt" ]] || {
  echo "Missing Keycloak CA. Run ./scripts/auth/setup.sh first." >&2
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

if [[ "$application_source" != image ]]; then
  [[ -f "$application_script" ]] || { echo "Missing Spark application script: $application_script" >&2; exit 1; }
fi

if [[ "$application_source" == configmap ]]; then
  [[ -n "$script_configmap" && -n "$script_mount_path" ]] || {
    echo "SPARK_APPLICATION_SCRIPT_CONFIGMAP and SPARK_APPLICATION_SCRIPT_MOUNT_PATH are required." >&2
    exit 1
  }
  kubectl -n "$job_namespace" create configmap "$script_configmap" \
    --from-file="$(basename "$application_script")=$application_script" \
    --dry-run=client -o yaml | kubectl apply -f - >/dev/null
fi

application_file_name="$(basename "$application_script")"
application_revision=""
if [[ "$application_source" == rustfs ]]; then
  [[ "$application_file_name" =~ ^[A-Za-z0-9._-]+$ ]] || {
    echo "Spark application file name may contain only letters, digits, dot, underscore, and hyphen." >&2
    exit 1
  }
  [[ "$artifact_bucket" =~ ^[a-z0-9][a-z0-9.-]*[a-z0-9]$ ]] || {
    echo "SPARK_APPLICATION_ARTIFACT_BUCKET is not a valid S3 bucket name." >&2
    exit 1
  }
  [[ "$artifact_prefix" =~ ^[A-Za-z0-9._/-]+$ && "$artifact_prefix" != /* ]] || {
    echo "SPARK_APPLICATION_ARTIFACT_PREFIX must be a relative object prefix." >&2
    exit 1
  }
  case "$artifact_url_expiration" in
    ''|*[!0-9]*) echo "SPARK_APPLICATION_URL_EXPIRATION must be an integer." >&2; exit 1 ;;
  esac
  (( artifact_url_expiration >= 1 && artifact_url_expiration <= 604800 )) || {
    echo "SPARK_APPLICATION_URL_EXPIRATION must be between 1 and 604800 seconds." >&2
    exit 1
  }

  s3_access_key="$(kubectl -n "$uc_namespace" get secret "$uc_credentials_secret" \
    -o jsonpath='{.data.S3_ACCESS_KEY}' | base64 --decode)"
  s3_secret_key="$(kubectl -n "$uc_namespace" get secret "$uc_credentials_secret" \
    -o jsonpath='{.data.S3_SECRET_KEY}' | base64 --decode)"
  [[ -n "$s3_access_key" && -n "$s3_secret_key" ]] || {
    echo "RustFS upload credentials are missing from $uc_namespace/$uc_credentials_secret." >&2
    exit 1
  }

  application_digest="$(openssl dgst -sha256 "$application_script" | awk '{print $NF}')"
  application_revision="$application_digest-$(date +%s)"
  artifact_key="${artifact_prefix%/}/$application_digest/$application_file_name"
  rustfs_upload_url="https://127.0.0.1:$rustfs_upload_port"
  kubectl -n "$rustfs_namespace" port-forward --address 127.0.0.1 \
    "service/$rustfs_service" "$rustfs_upload_port:9000" >/dev/null 2>&1 &
  rustfs_pid=$!
  for attempt in {1..60}; do
    if curl -fsS --cacert "$repo_root/k8s/storage/tls/ca.crt" \
      "$rustfs_upload_url/minio/health/live" >/dev/null 2>&1; then
      break
    fi
    if [[ "$attempt" -eq 60 ]]; then
      echo "RustFS is unavailable through service/$rustfs_service." >&2
      exit 1
    fi
    sleep 1
  done

  upload_status="$(curl -sS --cacert "$repo_root/k8s/storage/tls/ca.crt" \
    -o "$work_dir/rustfs-upload" -w '%{http_code}' -X PUT \
    --aws-sigv4 'aws:amz:us-east-1:s3' --user "$s3_access_key:$s3_secret_key" \
    --data-binary "@$application_script" \
    "$rustfs_upload_url/$artifact_bucket/$artifact_key")"
  [[ "$upload_status" == 200 ]] || {
    echo "Spark application upload failed (HTTP $upload_status): $(<"$work_dir/rustfs-upload")" >&2
    exit 1
  }

  application_url="$(
    AWS_ACCESS_KEY_ID="$s3_access_key" AWS_SECRET_ACCESS_KEY="$s3_secret_key" \
      python3 "$repo_root/scripts/storage/presign-s3-get.py" \
        --endpoint "https://$rustfs_service.$rustfs_namespace.svc.cluster.local:9000" \
        --bucket "$artifact_bucket" --key "$artifact_key" \
        --expires "$artifact_url_expiration"
  )"
  unset s3_access_key s3_secret_key
  kubectl -n "$job_namespace" create secret generic "$artifact_url_secret" \
    --from-literal=url="$application_url" \
    --dry-run=client -o yaml | kubectl apply -f - >/dev/null
  unset application_url
fi

java_home="$(java -XshowSettings:properties -version 2>&1 |
  sed -n 's/^[[:space:]]*java\.home = //p' | head -n 1)"
[[ -n "$java_home" && -f "$java_home/lib/security/cacerts" ]] || {
  echo "Cannot locate the active JRE truststore." >&2
  exit 1
}
cp "$java_home/lib/security/cacerts" "$work_dir/cacerts"
keytool -importcert -noprompt -storepass changeit -keystore "$work_dir/cacerts" \
  -alias mizuumi-rustfs -file "$repo_root/k8s/storage/tls/ca.crt" >/dev/null
keytool -importcert -noprompt -storepass changeit -keystore "$work_dir/cacerts" \
  -alias mizuumi-keycloak -file "$repo_root/k8s/auth/tls/ca.crt" >/dev/null

runtime_s3_access_key="$(kubectl -n "$uc_namespace" get secret "$uc_credentials_secret" \
  -o jsonpath='{.data.S3_ACCESS_KEY}' | base64 --decode)"
runtime_s3_secret_key="$(kubectl -n "$uc_namespace" get secret "$uc_credentials_secret" \
  -o jsonpath='{.data.S3_SECRET_KEY}' | base64 --decode)"
[[ -n "$runtime_s3_access_key" && -n "$runtime_s3_secret_key" ]] || {
  echo "RustFS credentials are missing from $uc_namespace/$uc_credentials_secret." >&2
  exit 1
}

credentials_file="$work_dir/credentials.env"
umask 077
printf 'KEYCLOAK_CLIENT_ID=%s\n' "$keycloak_client_id" > "$credentials_file"
printf 'KEYCLOAK_CLIENT_SECRET=%s\n' "$keycloak_client_secret" >> "$credentials_file"
printf 'UNITY_CATALOG_NAME=%s\n' "$catalog" >> "$credentials_file"
printf 'S3_ACCESS_KEY=%s\n' "$runtime_s3_access_key" >> "$credentials_file"
printf 'S3_SECRET_KEY=%s\n' "$runtime_s3_secret_key" >> "$credentials_file"
unset keycloak_client_secret runtime_s3_access_key runtime_s3_secret_key
kubectl -n "$job_namespace" create secret generic "$credentials_secret" \
  --from-env-file="$credentials_file" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

kubectl -n "$job_namespace" create secret generic "$trust_secret" \
  --from-file=cacerts="$work_dir/cacerts" \
  --from-file=keycloak-ca.crt="$repo_root/k8s/auth/tls/ca.crt" \
  --from-file=rustfs-ca.crt="$repo_root/k8s/storage/tls/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm repo add spark-operator https://kubeflow.github.io/spark-operator --force-update
helm repo update spark-operator
helm upgrade --install "$operator_release" spark-operator/spark-operator \
  --version "$operator_version" \
  --namespace "$operator_namespace" --create-namespace \
  --values "$repo_root/k8s/spark/operator-values.yaml" \
  --set "spark.jobNamespaces[0]=$job_namespace" \
  --wait --timeout 10m

application_options=()
if [[ "$application_source" == configmap ]]; then
  script_revision="$(cksum "$application_script" | awk '{print $1 "-" $2}')-$(date +%s)"
  application_options+=(
    --set-string "application.mainApplicationFile=local://$script_mount_path/$application_file_name"
    --set-string "application.scriptConfigMap=$script_configmap"
    --set-string "application.scriptMountPath=$script_mount_path"
    --set-string "application.scriptRevision=$script_revision"
  )
elif [[ "$application_source" == rustfs ]]; then
  application_options+=(
    --set-string "application.mainApplicationFile=local:///opt/spark/work-dir/application/$application_file_name"
    --set "application.artifact.enabled=true"
    --set-string "application.artifact.urlSecretName=$artifact_url_secret"
    --set-string "application.artifact.fileName=$application_file_name"
    --set-string "application.scriptRevision=$application_revision"
  )
fi

# Bash 3.2 with nounset rejects a direct expansion when this array is empty.
helm upgrade --install "$application_release" "$repo_root/k8s/spark" \
  --namespace "$job_namespace" \
  --set-string "image.repository=$image_repository" \
  --set-string "image.tag=$image_tag" \
  --set-string "credentialsSecretName=$credentials_secret" \
  --set-string "trustSecretName=$trust_secret" \
  --set-string "spark.catalog.name=$catalog" \
  --set-string "spark.catalog.storageRoot=$storage_root" \
  ${application_options[@]+"${application_options[@]}"} \
  "$@"

echo "Spark Operator and Unity Catalog / RustFS example are installed."
echo "Watch: kubectl -n $job_namespace get sparkapplications -w"
echo "Logs:  kubectl -n $job_namespace logs -l spark-role=driver --tail=-1"
