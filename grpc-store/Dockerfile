FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

WORKDIR /context/app


FROM chef AS planner
COPY ./grpc-store .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 

WORKDIR /context/app
COPY ./grpc-codegen ../grpc-codegen
COPY --from=planner /context/app/recipe.json recipe.json
RUN apt-get update && \
    apt-get install -y protobuf-compiler
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./grpc-store .
RUN  cargo build --release --bin grpc-store

FROM debian:bookworm-slim AS runtime

WORKDIR /context/app
COPY ./self-signed-certs /self-signed-certs
COPY --from=builder /context/app/target/release/grpc-store /usr/local/bin
ENTRYPOINT ["/usr/local/bin/grpc-store"]