# fakelogs
A crazy fast fake log data generator

# Usage

## Config file format

```yaml
---
# Patterns to generate logs from.
patterns:
  - '{"name":"{username}", "age":"{int}", "Moto": "{sentence}", "Summary": "{paragraph}", "ip_addr": "{ipv4}", "email": "{email}", "timestamp":"{ts_rfc3339}"}'

  # Optional: a weight sets how often a pattern is used relative to the others (default 1).
  # Here the access line is 19 times as common as the error line. A weight of 0 turns a pattern off.
  - pattern: '{ipv4} - - "GET /{word} HTTP/1.1" 200'
    weight: 19
  - pattern: '{ipv4} - - "GET /{word} HTTP/1.1" 500'

# Maximum number of log events per second.
# Set to 0 for uncapped EPS. Defaults to 10 if left out.
eps: 10
```

Weights are exact, not random: with weights 19 and 1, every 20 events contain exactly 19 of the first pattern and 1 of the second, spread out evenly. Patterns with equal weights are used in turn, in the order they are listed.

## Placeholders

| Placeholder | Example |
|---|---|
| `{ts_rfc3339}` | `2026-10-24T02:55:22.024855810+00:00` |
| `{year}` `{month}` `{day}` `{hour}` `{hour12}` `{minute}` `{second}` | parts of the current UTC time, zero padded: `2026` `10` `04` `09` `09` `05` `07` |
| `{month_name}` | `Oct` |
| `{ipv4}` | `204.131.224.229` |
| `{ipv6}` | `9ebd:7ec9:8b5d:9654:7e1d:92ee:7d22:2aa0` |
| `{uuid}` | `1130b3d0-4db6-4b58-a099-309875054e6a` |
| `{hex:N}` | N random lowercase hex digits, e.g. `{hex:8}` → `9f86d081` |
| `{unix}` | seconds since the Unix epoch, e.g. `1792810540` |
| `{username}` | random first name |
| `{email}` | `Joyelle@BZYHO.co.uk` |
| `{domain}` `{tld}` | `BZYHO` `co.uk` |
| `{str}` | 10-11 random uppercase letters |
| `{int}` | number from 1 to 99 |
| `{int:MIN-MAX}` | number from MIN to MAX, both included, e.g. `{int:1024-65535}` |
| `{choice:A\|B\|C}` | one of the listed values, e.g. `{choice:GET\|POST}`; repeat a value to make it more likely, leave one empty for "sometimes nothing". Placeholders can't be nested inside a choice |
| `{word}` `{sentence}` `{paragraph}` | random words, 5-14 words, 5-14 sentences |

Anything else in braces, like the braces of a JSON object, is copied through as-is. All time placeholders in one event share the same timestamp.
See [example/mixed.yaml](example/mixed.yaml) for patterns in several common log formats.

## Example library

Ready-to-run configs for common log formats live in [example/](example/). Run one with:

```shell
fakelogs generate -c example/nginx-access.yaml
```

**Web servers and proxies**

| File | Format |
|---|---|
| [nginx-access.yaml](example/nginx-access.yaml) | nginx / Apache combined access log |
| [nginx-error.yaml](example/nginx-error.yaml) | nginx error log |
| [haproxy.yaml](example/haproxy.yaml) | HAProxy HTTP and TCP logs, health check events |
| [envoy-istio.yaml](example/envoy-istio.yaml) | Envoy and Istio sidecar access logs |
| [traefik.yaml](example/traefik.yaml) | Traefik JSON access log |
| [caddy.yaml](example/caddy.yaml) | Caddy JSON access and error log |
| [iis-w3c.yaml](example/iis-w3c.yaml) | Microsoft IIS, W3C Extended format |

**Cloud**

| File | Format |
|---|---|
| [aws-alb.yaml](example/aws-alb.yaml) | AWS Application Load Balancer access log |
| [aws-cloudfront.yaml](example/aws-cloudfront.yaml) | Amazon CloudFront standard log (tab separated) |
| [aws-s3-access.yaml](example/aws-s3-access.yaml) | Amazon S3 server access log |
| [aws-vpc-flow.yaml](example/aws-vpc-flow.yaml) | AWS VPC flow log (version 2) |
| [aws-cloudtrail.yaml](example/aws-cloudtrail.yaml) | AWS CloudTrail: sign-ins, AssumeRole, S3, IAM, AccessDenied |
| [gcp-cloud-logging.yaml](example/gcp-cloud-logging.yaml) | Google Cloud Logging: GKE, load balancer, audit logs |
| [azure-activity.yaml](example/azure-activity.yaml) | Azure Activity Log |

**Kubernetes and observability**

| File | Format |
|---|---|
| [k8s-container-cri.yaml](example/k8s-container-cri.yaml) | Container log lines in the CRI format, as on Kubernetes nodes |
| [k8s-audit.yaml](example/k8s-audit.yaml) | Kubernetes API server audit log |
| [otel-logs.yaml](example/otel-logs.yaml) | OpenTelemetry log records in OTLP/JSON, readable by the Collector's `otlp_json_file` receiver |

**Security**

| File | Format |
|---|---|
| [syslog-rfc3164.yaml](example/syslog-rfc3164.yaml) | BSD syslog: sshd, sudo, cron, systemd, kernel |
| [syslog-rfc5424.yaml](example/syslog-rfc5424.yaml) | IETF syslog with structured data |
| [linux-auditd.yaml](example/linux-auditd.yaml) | Linux auditd: syscalls, logins, sudo, SELinux denials |
| [windows-security.yaml](example/windows-security.yaml) | Windows Security events 4624, 4625, 4672, 4688, 4720, 4740 as JSON |
| [zeek-conn.yaml](example/zeek-conn.yaml) | Zeek conn.log (tab separated) |
| [zeek-dns.yaml](example/zeek-dns.yaml) | Zeek dns.log (tab separated) |
| [suricata-eve.yaml](example/suricata-eve.yaml) | Suricata EVE JSON: alerts, DNS, HTTP, TLS, flows |
| [cef-firewall.yaml](example/cef-firewall.yaml) | Firewall events in ArcSight CEF |
| [okta-system-log.yaml](example/okta-system-log.yaml) | Okta System Log: sign-ins, MFA, SSO, group changes |
| [github-audit.yaml](example/github-audit.yaml) | GitHub organization audit log |

**Databases and middleware**

| File | Format |
|---|---|
| [postgresql.yaml](example/postgresql.yaml) | PostgreSQL server log |
| [mysql-slow.yaml](example/mysql-slow.yaml) | MySQL slow query log (multi-line) |
| [mysql-error.yaml](example/mysql-error.yaml) | MySQL 8 error log |
| [mongodb.yaml](example/mongodb.yaml) | MongoDB structured JSON log |
| [redis.yaml](example/redis.yaml) | Redis server log |
| [kafka.yaml](example/kafka.yaml) | Apache Kafka broker log |

**Application logs**

| File | Format |
|---|---|
| [json-app.yaml](example/json-app.yaml) | JSON application logs |
| [logfmt-app.yaml](example/logfmt-app.yaml) | logfmt (key=value) application logs |
| [logstash-json.yaml](example/logstash-json.yaml) | logstash-logback-encoder JSON (Java/Spring) |
| [ecs.yaml](example/ecs.yaml) | Elastic Common Schema (ECS) JSON |
| [gelf.yaml](example/gelf.yaml) | Graylog GELF 1.1 |
| [multiline-stacktraces.yaml](example/multiline-stacktraces.yaml) | Java and Python stack traces mixed with normal lines |
| [mixed.yaml](example/mixed.yaml) | One of each common style, using every placeholder |

Each placeholder is random on its own, so related fields aren't linked: for example, an ALB entry's `elb_status_code` and `target_status_code` can differ. Where fields must agree (like an event ID and its message), the examples use one pattern per case.

### Multi-line and tab-separated patterns

Write a pattern in double quotes to use `\n` (new line) and `\t` (tab), as in [multiline-stacktraces.yaml](example/multiline-stacktraces.yaml) and [zeek-conn.yaml](example/zeek-conn.yaml):

```yaml
patterns:
  - "{ts_rfc3339} ERROR request failed\njava.lang.IllegalStateException: {sentence}\n\tat com.example.Service.run(Service.java:{int:10-400})"
```

## Generate logs and output to STDOUT

```shell
fakelogs generate -c </path/to/config/file.yaml>
```

`--eps <n>` overrides the config file's rate (0 for uncapped):

```shell
fakelogs generate -c example/nginx-access.yaml --eps 500
```

## Generate logs without a config file

Give patterns on the command line with `-p` (repeat it for more patterns). Without `--eps`, the rate is 10 events per second.

```shell
fakelogs generate -p '{ts_rfc3339} user={username} ip={ipv4}' --eps 100
fakelogs generate -p 'GET /{word} 200' -p 'GET /{word} 500' --eps 0
```

With `-c` as well, `-p` replaces the config file's patterns but keeps its `eps`.

## Generate a fixed number of events

`-n <count>` stops after that many events, which is handy for writing test files:

```shell
fakelogs generate -c example/nginx-access.yaml --eps 0 -n 1000000 > access.log
```

## Repeatable output

`--seed <n>` makes the random values repeatable: the same seed, patterns and `--threads` give the same output every run. Time placeholders (`{ts_rfc3339}`, `{unix}`, `{hour}` and so on) still come from the clock.

```shell
fakelogs generate -p '{ipv4} {username} {uuid}' -n 5 --seed 42
```

With more than one thread, the same lines are generated but their order can vary.

## Use multiple CPU cores

Both `generate` and `benchmark` take `--threads <n>` (default 1). Use `--threads 0` for one thread per CPU.
With an EPS cap, the rate is shared across the threads, so the total still matches the cap.

```shell
fakelogs benchmark -c </path/to/config/file.yaml> --threads 8

Benchmarking with 8 thread(s) ...
100000 log events generated in 0.099941843 seconds
EPS: 1000581
```

## Benchmark generating 100K logs with uncapped EPS

```shell
fakelogs benchmark -c </path/to/config/file.yaml>

Benchmarking with 1 thread(s) ...
100000 log events generated in 0.715620347 seconds
EPS: 139739
```

## Benchmark one second of events at a capped rate

`--eps <n>` benchmarks `n` events at `n` per second. `--eps` with no value uses the config file's rate.

```shell
fakelogs benchmark -c </path/to/config/file.yaml> --eps 1500

Benchmarking with 1 thread(s) ...
1500 log events generated in 1.000130086 seconds
EPS: 1500
```