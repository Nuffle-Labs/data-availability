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
    cbindgen \
&& rm -rf /var/lib/apt/lists/*

COPY . /app

WORKDIR /app

RUN mkdir -p lib

RUN make da-rpc-sys

RUN cp -f ./target/release/libnear* ./lib/

RUN ls ./lib

RUN rm -rf ./target
RUN rm -rf ~/.cargo

RUN ls -lh /home
