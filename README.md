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

---

### Routing Priority

1. Query (?view=mobile|desktop)
2. Cookie override
3. User-Agent detection

---

### Observability

- Prometheus metrics (/metrics)
- JSON structured logs (tracing)
- device-level traffic breakdown
- per-host statistics

---

## Usage

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

---

### Example

```bash
curl -H "Host: example.com" \
 -H "User-Agent: Mozilla/5.0 (iPhone)" \
 http://localhost:8080/path

Response:

HTTP/1.1 302 Found
Location: https://m.example.com/path
```

---

### Metrics endpoint

```text
GET /metrics

Example output:

redirect_total{device="mobile",host="example.com"} 1234
redirect_total{device="desktop",host="example.com"} 987
```

---

### Logging

```text
Logs are emitted in JSON format:

{
"host": "example.com",
"device": "mobile",
"ua": "Mozilla/5.0 ...",
"target_url": "https://m.example.com/path",
"event": "redirect"
}
```

---

### Prometheus Integration

scrape_configs:

- job_name: "domain-redirector"
  static_configs:
  - targets: ["domain-redirector:8080"]

---
