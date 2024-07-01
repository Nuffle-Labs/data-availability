FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /near

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /near/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin near-da-sidecar 

RUN ldd target/release/near-da-sidecar

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.source https://github.com/nuffle-labs/data-availability

WORKDIR /app

RUN apt-get update && apt-get install -y \
  ca-certificates

COPY --from=builder /near/target/release/near-da-sidecar /usr/local/bin
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/near-da-sidecar"]
