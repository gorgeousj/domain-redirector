# syntax=docker/dockerfile:1
## Runtime Stage
FROM alpine:3.23

RUN apk add --no-cache ca-certificates tzdata
ENV TZ=UTC

RUN addgroup -g 10001 app \
    && adduser -D -u 10001 -G app -h /app app

WORKDIR /app

ENV SERVICE_PORT=8080

COPY --chown=app:app ./target/x86_64-unknown-linux-musl/release/domain-redirector .
COPY --chown=app:app ./config.toml .
RUN chmod +x /app/domain-redirector

USER 10001

EXPOSE ${SERVICE_PORT}

CMD ["/app/domain-redirector", "/app/config.toml"]
