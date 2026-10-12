# syntax=docker/dockerfile:1.7

FROM node:22-bookworm-slim AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY tsconfig.json ./
COPY src ./src
RUN npm run build

FROM node:22-bookworm-slim AS runner
RUN groupadd --system --gid 65532 synthetics \
  && useradd --system --uid 65532 --gid synthetics --home-dir /home/synthetics --create-home synthetics
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --omit=dev --no-audit --no-fund \
  && npm cache clean --force
COPY --from=builder --chown=65532:65532 /app/dist ./dist
USER 65532:65532
ENV NODE_ENV=production \
  HOST=0.0.0.0 \
  PORT=4020
EXPOSE 4020
ENTRYPOINT ["node", "dist/server.js"]
