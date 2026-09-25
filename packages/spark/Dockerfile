# syntax=docker/dockerfile:1

ARG SPARK_VERSION=4.0.1
FROM apache/spark:${SPARK_VERSION}-scala2.13-java17-python3-ubuntu

LABEL org.opencontainers.image.title="mizuumi-engine Spark" \
      org.opencontainers.image.description="PySpark runtime for Unity Catalog and RustFS" \
      org.opencontainers.image.source="https://github.com/linrium/mizuumi-engine"

USER root
COPY --chown=185:0 examples /opt/spark/work-dir/examples
USER 185
