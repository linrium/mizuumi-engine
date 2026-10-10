ARG PYTHON_VERSION=3.13

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
COPY --chown=duckdb:duckdb server.py /app/server.py

USER duckdb

# Bake the extensions into the image so the pod does not need internet access.
RUN python -c "import duckdb; c=duckdb.connect(); c.execute('INSTALL httpfs'); c.execute('INSTALL quack')"

EXPOSE 9494

ENTRYPOINT ["python", "/app/server.py"]
