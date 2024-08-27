# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.79.0
ARG APP_NAME=urlshortener

FROM rust:${RUST_VERSION}-alpine AS build_test
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git jq

COPY env.dev env.dev 
COPY env.test env.test 

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --bins --tests --message-format=json -q > build_output && \
    cp `cat build_output | jq -r 'select(.target.kind[0] == "bin") | select(.profile.test == false) | .executable'` /bin/server && \
    cp `cat build_output | jq -r 'select(.target.kind[0] == "bin") | select(.profile.test == true) | .executable'` /bin/tests
