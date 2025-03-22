# Alpine is broke, so we're using Debian
FROM rust:slim

ARG DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install -y pkgconf libzmq3-dev libssl-dev gcc-multilib

COPY . /build/edtear
WORKDIR /build/edtear

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    SQLX_OFFLINE=true cargo install --path .

ENV RUST_LOG=info
ARG RUST_LOG=info
# ENTRYPOINT ["tail", "-f", "/dev/null"]
ARG POSTGRES_PASSWORD="password"
ENTRYPOINT edtear listen --url postgres://postgres:${POSTGRES_PASSWORD}@edtear-postgres-1:5432/edtear
