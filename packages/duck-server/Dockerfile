ARG PYTHON_VERSION=3.13
ARG DUCKDB_VERSION=1.5.3

FROM rust:bookworm AS unity-catalog-builder

ARG DUCKDB_VERSION

RUN apt-get update \
  && apt-get install --no-install-recommends -y cmake ninja-build python3 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /src/duck-uc-extension
COPY duck-uc-extension/ ./

# Build the repository fork against the exact DuckDB release used at runtime.
# The copied fork has no usable Git metadata, so provide the version explicitly.
RUN make release GEN=ninja BUILD_EXTENSION_TEST_DEPS=none \
  OVERRIDE_GIT_DESCRIBE="v${DUCKDB_VERSION}-0-g0123456789"

FROM python:${PYTHON_VERSION}-slim

ARG DUCKDB_VERSION=1.5.3

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
