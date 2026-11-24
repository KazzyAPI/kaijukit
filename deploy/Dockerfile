FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY migrations migrations
RUN cargo build --release -p kk-server

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 kk
COPY --from=build /src/target/release/kk-server /usr/local/bin/kk-server
USER kk
EXPOSE 8080
ENTRYPOINT ["kk-server"]
