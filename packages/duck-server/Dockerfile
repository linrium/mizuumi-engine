ARG PYTHON_VERSION=3.13
ARG DUCKDB_VERSION=1.5.6
ARG UNITY_CATALOG_COMMIT=02024095fe6eeed6f53e78dce43e83c7424abec2
ARG VCPKG_COMMIT=cd61e1e26a038e82d6550a3ebbe0fbbfe7da78e3

FROM rust:bookworm AS unity-catalog-builder

ARG DUCKDB_VERSION
ARG UNITY_CATALOG_COMMIT
ARG VCPKG_COMMIT

RUN apt-get update \
  && apt-get install --no-install-recommends -y cmake ninja-build python3 pkg-config zip unzip \
  && rm -rf /var/lib/apt/lists/*

RUN git clone https://github.com/microsoft/vcpkg.git /opt/vcpkg \
  && git -C /opt/vcpkg checkout --detach "$VCPKG_COMMIT" \
  && /opt/vcpkg/bootstrap-vcpkg.sh -disableMetrics

ENV VCPKG_ROOT=/opt/vcpkg \
  VCPKG_TOOLCHAIN_PATH=/opt/vcpkg/scripts/buildsystems/vcpkg.cmake \
  USE_MERGED_VCPKG_MANIFEST=1

WORKDIR /src/duck-uc-extension
RUN git init \
  && git remote add origin https://github.com/duckdb/unity_catalog.git \
  && git fetch --depth 1 origin "$UNITY_CATALOG_COMMIT" \
  && git checkout --detach FETCH_HEAD \
  && git submodule update --init extension-ci-tools

COPY duck-server/patches/unity-catalog-custom-s3.patch /tmp/unity-catalog-custom-s3.patch
RUN git apply /tmp/unity-catalog-custom-s3.patch

# The fork carries a DuckDB checkout from a different revision. Replacing it is
# required: changing only the extension metadata makes an ABI-incompatible
# binary that can crash the runtime while loading.
RUN git clone --branch "v${DUCKDB_VERSION}" --depth 1 \
    https://github.com/duckdb/duckdb.git /tmp/duckdb \
  && rm -rf duckdb \
  && mv /tmp/duckdb duckdb

# Build the repository fork against the exact DuckDB release used at runtime.
RUN make release GEN=ninja BUILD_EXTENSION_TEST_DEPS=none \
  OVERRIDE_GIT_DESCRIBE="v${DUCKDB_VERSION}"

FROM python:${PYTHON_VERSION}-slim

ARG DUCKDB_VERSION=1.5.6

RUN groupadd --system --gid 65532 duckdb \
  && useradd --system --uid 65532 --gid duckdb --home-dir /home/duckdb --create-home duckdb \
  && mkdir -p /var/lib/duckdb \
  && chown duckdb:duckdb /var/lib/duckdb \
  && pip install --no-cache-dir "duckdb==${DUCKDB_VERSION}"

ENV HOME=/home/duckdb \
  PYTHONUNBUFFERED=1

WORKDIR /app
COPY --from=unity-catalog-builder \
  /src/duck-uc-extension/build/release/extension/unity_catalog/unity_catalog.duckdb_extension \
  /opt/duckdb/extensions/unity_catalog.duckdb_extension
COPY --from=unity-catalog-builder \
  /src/duck-uc-extension/build/release/extension/delta/delta.duckdb_extension \
  /opt/duckdb/extensions/delta.duckdb_extension
COPY --chown=duckdb:duckdb duck-server/server.py /app/server.py

USER duckdb

# Unity Catalog and Delta above are built from the repository fork.
RUN python -c "import duckdb; c=duckdb.connect(); c.execute('INSTALL httpfs'); c.execute('INSTALL quack')"

EXPOSE 9494

ENTRYPOINT ["python", "/app/server.py"]
