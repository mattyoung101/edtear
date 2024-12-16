# Alpine is broke, so we're using Debian
FROM rust:slim

ARG DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install -y pkgconf libzmq3-dev libssl-dev gcc-multilib

COPY . /build/edtear
WORKDIR /build/edtear

RUN SQLX_OFFLINE=true cargo install --path .

ENV RUST_LOG=info
ARG RUST_LOG=info
# ENTRYPOINT ["tail", "-f", "/dev/null"]
ENTRYPOINT ["edtear", "listen", "--url", "postgres://postgres:password@edtear-postgres-1:5432/edtear"]
