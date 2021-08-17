FROM debian:11 as build-rs

RUN apt-get update
RUN apt-get install -y \
  curl \
  musl-tools

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/local/src/va-tool

RUN mkdir -pv src/
RUN echo "fn main() {println!(\"---\")}" > ./src/main.rs
COPY ./Cargo.toml ./
RUN RUSTFLAGS=-Clinker=musl-gcc cargo \
  build \
  --release \
  --verbose \
  --target=x86_64-unknown-linux-musl

RUN rm -rvf ./src/*

COPY ./src/ ./src/
RUN RUSTFLAGS=-Clinker=musl-gcc cargo \
  build \
  --release \
  --verbose \
  --target=x86_64-unknown-linux-musl

RUN strip ./target/x86_64-unknown-linux-musl/release/va-tool

# -------------------------------------
FROM alpine:latest

COPY --from=build-rs /usr/local/src/va-tool/target/x86_64-unknown-linux-musl/release/va-tool /usr/local/bin/va-tool
