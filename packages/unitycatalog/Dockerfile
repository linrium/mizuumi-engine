# syntax=docker/dockerfile:1

ARG UNITYCATALOG_VERSION=v0.6.0
FROM unitycatalog/unitycatalog:${UNITYCATALOG_VERSION}

LABEL org.opencontainers.image.title="mizuumi-engine Unity Catalog" \
      org.opencontainers.image.description="Pinned Unity Catalog OSS server runtime" \
      org.opencontainers.image.source="https://github.com/unitycatalog/unitycatalog"

EXPOSE 8080
VOLUME ["/home/unitycatalog/etc/db"]

HEALTHCHECK --interval=10s --timeout=5s --start-period=60s --retries=12 \
  CMD wget -q -O /dev/null http://127.0.0.1:8080/api/2.1/unity-catalog/catalogs || exit 1

