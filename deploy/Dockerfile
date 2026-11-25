FROM node:20-bookworm-slim AS web
RUN corepack enable
WORKDIR /src/web
COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml ./
COPY web/app/package.json app/
COPY web/packages/ui/package.json packages/ui/
COPY web/packages/editor/package.json packages/editor/
RUN pnpm install --frozen-lockfile
COPY web ./
RUN pnpm --filter app build

FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY migrations migrations
RUN cargo build --release -p kk-server

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 kk && mkdir -p /data/blobs && chown kk /data/blobs
COPY --from=build /src/target/release/kk-server /usr/local/bin/kk-server
COPY --from=web /src/web/app/dist /srv/web
ENV KK_WEB_DIR=/srv/web
USER kk
EXPOSE 8080
ENTRYPOINT ["kk-server"]
