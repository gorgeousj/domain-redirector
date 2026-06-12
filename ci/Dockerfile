# syntax=docker/dockerfile:1
## Build Stage
FROM rust:1.96-alpine3.23 AS builder

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /build

RUN apk add --no-cache \
    musl-dev \
    gcc \
    g++ \
    make \
    openssl-dev \
    pkgconfig

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src && echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --target x86_64-unknown-linux-musl

RUN rm -rf src

## building
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --target x86_64-unknown-linux-musl


## Runtime Stage
FROM alpine:3.23

RUN apk add --no-cache ca-certificates tzdata
ENV TZ=UTC

RUN addgroup -g 10001 app \
    && adduser -D -u 10001 -G app -h /app app

WORKDIR /app

ENV SERVICE_PORT=8080

COPY --from=builder --chown=app:app /build/target/x86_64-unknown-linux-musl/release/domain-redirector .
COPY config.toml .
RUN chmod +x /app/domain-redirector

USER 10001

EXPOSE ${SERVICE_PORT}

CMD ["/app/domain-redirector", "/app/config.toml"]
