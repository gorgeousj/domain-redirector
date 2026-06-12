# domain-redirector

A high-performance domain mobile/desktop redirect service built with Rust (Axum).

It performs intelligent redirection based on:

- User-Agent
- Cookie override
- Query override
- Host normalization

and exposes Prometheus metrics + structured JSON logs for production observability.

---

## Use Cases

- Mobile / Desktop domain splitting
- Multi-tenant domain routing
- A/B testing landing pages
- Edge redirect gateway

---

## Features

### Core routing logic

- Mobile / Desktop domain switching
- Cookie-based override
- Query parameter override (?view=mobile|desktop)
- Host normalization (supports reverse proxy headers)
- Loop protection (avoid recursive redirects)
- Path + query preservation

### Routing Priority

1. Query (?view=mobile|desktop)
2. Cookie override
3. User-Agent detection

### Observability

- Prometheus metrics (/metrics)
- JSON structured logs (tracing)
- device-level traffic breakdown
- per-host statistics

---

## Usage

### Building

```bash
# building with zigbuild
cargo zigbuild --release \
    --target x86_64-unknown-linux-musl

# building container
docker buildx build --platform linux/amd64 -t domain-redirector:0.1.0 . -f ci/Dockerfile

```

### Configuration

```toml
# config.toml
listen = "0.0.0.0:8080"

mobile_prefix = "m"
desktop_prefix = "www"

redirect_code = 302

cookie_name = "site"

mobile_cookie_value = "mobile"
desktop_cookie_value = "desktop"
```

### Example

```bash
# PC/Laptop
curl -i -H "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 15_0)" http://example.com/news?id=1

Response:
HTTP/1.1 302 Found
Location: https://www.example.com/news?id=1

# Mobile/Pad
curl -i -H "User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X)" http://example.com/news?id=1

Response:
HTTP/1.1 302 Found
Location: https://m.example.com/news?id=1

```

### Metrics endpoint

```text
GET /metrics

Example output:

redirect_total{device="mobile",host="example.com"} 1234
redirect_total{device="desktop",host="example.com"} 987
```

### Logging

```json
Logs are emitted in JSON format:

{
"host": "example.com",
"device": "mobile",
"ua": "Mozilla/5.0 ...",
"target_url": "https://m.example.com/path",
"event": "redirect"
}
```

### Prometheus Integration

```yaml
scrape_configs:
  - job_name: "domain-redirector"
    static_configs:
      - targets: ["domain-redirector:8080"]
```

---
