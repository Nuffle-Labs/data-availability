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

RUN cp -f ./target/release/libnear* /lib/
RUN ls /lib

RUN mkdir -p /gopkg
RUN cp -rf ./gopkg/* /gopkg
RUN mkdir -p /gopkg/da-rpc/lib
RUN cp -f /lib/libnear* /gopkg/da-rpc/lib
RUN ls /gopkg
RUN ls /gopkg/da-rpc/lib


WORKDIR /lib 

RUN rm -rf /app
RUN rm -rf ~/.cargo
