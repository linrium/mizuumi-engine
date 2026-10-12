# syntax=docker/dockerfile:1

ARG SPARK_VERSION=4.0.1
FROM apache/spark:${SPARK_VERSION}-scala2.13-java17-python3-ubuntu

ARG DELTA_VERSION=4.3.1
ARG HADOOP_AWS_VERSION=3.4.1
ARG UNITY_CATALOG_VERSIONS="0.5.0 0.6.0"
ARG KAFKA_CONNECTOR_VERSION=4.0.1

LABEL org.opencontainers.image.title="mizuumi-engine Spark base" \
      org.opencontainers.image.description="Reusable PySpark runtime with cached Unity Catalog and RustFS dependencies" \
      org.opencontainers.image.source="https://github.com/linrium/mizuumi-engine"

USER root
COPY dependency_warmup.py /tmp/dependency_warmup.py
RUN ivy_dir=/tmp/spark-ivy && \
    dependency_root=/opt/spark/dependencies && \
    mkdir -p "$ivy_dir" "$dependency_root/jars" && \
    for unity_catalog_version in ${UNITY_CATALOG_VERSIONS}; do \
      rm -rf "$ivy_dir/jars" && \
      /opt/spark/bin/spark-submit \
        --master 'local[1]' \
        --conf "spark.jars.ivy=$ivy_dir" \
        --packages "io.delta:delta-spark_4.0_2.13:${DELTA_VERSION},io.unitycatalog:unitycatalog-spark_4.0_2.13:${unity_catalog_version},org.apache.hadoop:hadoop-aws:${HADOOP_AWS_VERSION}" \
        /tmp/dependency_warmup.py && \
      mkdir -p "$dependency_root/$unity_catalog_version" && \
      for jar in "$ivy_dir"/jars/*.jar; do \
        jar_name="$(basename "$jar")"; \
        if [ ! -e "$dependency_root/jars/$jar_name" ]; then \
          mv "$jar" "$dependency_root/jars/$jar_name"; \
        fi; \
        ln -s "../jars/$jar_name" "$dependency_root/$unity_catalog_version/$jar_name"; \
      done; \
    done && \
    rm -rf "$ivy_dir/jars" && \
    /opt/spark/bin/spark-submit \
      --master 'local[1]' \
      --conf "spark.jars.ivy=$ivy_dir" \
      --packages "org.apache.spark:spark-sql-kafka-0-10_2.13:${KAFKA_CONNECTOR_VERSION}" \
      /tmp/dependency_warmup.py && \
    for jar in "$ivy_dir"/jars/*.jar; do \
      jar_name="$(basename "$jar")"; \
      if [ ! -e "$dependency_root/jars/$jar_name" ]; then \
        mv "$jar" "$dependency_root/jars/$jar_name"; \
      fi; \
      for unity_catalog_version in ${UNITY_CATALOG_VERSIONS}; do \
        if [ ! -e "$dependency_root/$unity_catalog_version/$jar_name" ]; then \
          ln -s "../jars/$jar_name" "$dependency_root/$unity_catalog_version/$jar_name"; \
        fi; \
      done; \
    done && \
    rm -rf /tmp/dependency_warmup.py "$ivy_dir"
USER 185
