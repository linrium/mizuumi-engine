# syntax=docker/dockerfile:1.7

FROM node:22-bookworm-slim AS builder
WORKDIR /app
RUN apt-get update \
  && apt-get install -y --no-install-recommends python3 make g++ \
  && rm -rf /var/lib/apt/lists/*
COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY tsconfig.json ./
COPY src ./src
RUN npm run build \
  && npm prune --omit=dev \
  && npm cache clean --force

FROM node:22-bookworm-slim AS runner
RUN groupadd --system --gid 65532 synthetics \
  && useradd --system --uid 65532 --gid synthetics --home-dir /home/synthetics --create-home synthetics
WORKDIR /app
COPY package.json package-lock.json ./
COPY --from=builder --chown=65532:65532 /app/node_modules ./node_modules
COPY --from=builder --chown=65532:65532 /app/dist ./dist
USER 65532:65532
ENV NODE_ENV=production \
  HOST=0.0.0.0 \
  PORT=4020
EXPOSE 4020
ENTRYPOINT ["node", "dist/server.js"]
