FROM --platform=$BUILDPLATFORM rust:1.72.0-slim-bookworm

RUN apt-get update && apt-get install -y \
    git \
    jq \
    make \
    bash \
    openssl \
    libssl-dev \
    protobuf-compiler \
    pkg-config \
    cbindgen

COPY . /app

WORKDIR /app

RUN make op-rpc-sys

RUN rm -rf /app/target

RUN mkdir -p lib
RUN cp -f ./target/release/libnear* ./lib/
RUN ls ./lib
