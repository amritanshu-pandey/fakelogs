# fakelogs
A crazy fast fake log data generator

# Usage

## Config file format

```yaml
---
# List of patterns to generate logs for. Currently logs are generated for all specified patterns evenly.
patterns:
  - '{"name":"{username}", "age":"{int}", "Moto": "{sentence}", "Summary": "{paragraph}", "ip_addr": "{ipv4}", "email": "{email}", "timestamp":"{ts_rfc3339}"}'

# Maximum number of log events per second.
# Set to 0 for generating uncapped EPS
eps: 10
```

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

Ready-to-run configs for common log formats live in [example/](example/):

| File | Format |
|---|---|
| [nginx-access.yaml](example/nginx-access.yaml) | nginx / Apache combined access log |
| [nginx-error.yaml](example/nginx-error.yaml) | nginx error log |
| [haproxy.yaml](example/haproxy.yaml) | HAProxy HTTP and TCP logs, health check events |
| [aws-alb.yaml](example/aws-alb.yaml) | AWS Application Load Balancer access log |
| [aws-vpc-flow.yaml](example/aws-vpc-flow.yaml) | AWS VPC flow log (version 2) |
| [syslog-rfc3164.yaml](example/syslog-rfc3164.yaml) | BSD syslog: sshd, sudo, cron, systemd, kernel |
| [syslog-rfc5424.yaml](example/syslog-rfc5424.yaml) | IETF syslog with structured data |
| [json-app.yaml](example/json-app.yaml) | JSON application logs |
| [logfmt-app.yaml](example/logfmt-app.yaml) | logfmt (key=value) application logs |
| [postgresql.yaml](example/postgresql.yaml) | PostgreSQL server log |
| [cef-firewall.yaml](example/cef-firewall.yaml) | Firewall events in ArcSight CEF |
| [mixed.yaml](example/mixed.yaml) | One of each, using every placeholder |

```shell
fakelogs generate -c example/nginx-access.yaml
```

Each placeholder is random on its own, so related fields aren't linked: for example, an ALB entry's `elb_status_code` and `target_status_code` can differ.

## Generate logs and output to STDOUT

```shell
fakelogs generate -c </path/to/config/file.yaml>
```

## Use multiple CPU cores

Both `generate` and `benchmark` take `--threads <n>` (default 1). Use `--threads 0` for one thread per CPU.
With an EPS cap, the rate is shared across the threads, so the total still matches the config file.

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

## Benchmark using the EPS value specified in config file

```shell
fakelogs benchmark -c </path/to/config/file.yaml> --eps

Benchmarking with 1 thread(s) ...
1500 log events generated in 1.000130086 seconds
EPS: 1500
```