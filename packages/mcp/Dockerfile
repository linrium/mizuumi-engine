FROM node:22-bookworm-slim AS builder
WORKDIR /app
COPY mcp/package.json mcp/package-lock.json* ./
RUN npm install --no-audit --no-fund
COPY mcp/tsconfig.json ./tsconfig.json
COPY mcp/src ./src
RUN npx tsc

FROM node:22-bookworm-slim
RUN groupadd --system --gid 65532 mcp \
  && useradd --system --uid 65532 --gid mcp --home-dir /home/mcp --create-home mcp
WORKDIR /app
COPY mcp/package.json mcp/package-lock.json* ./
RUN npm install --omit=dev --no-audit --no-fund \
  && npm cache clean --force
COPY --from=builder /app/dist ./dist
USER mcp
ENV NODE_ENV=production \
  HOST=0.0.0.0 \
  PORT=8080
EXPOSE 8080
ENTRYPOINT ["node", "dist/server.js"]
