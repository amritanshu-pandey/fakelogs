# fakelogs
A crazy fast fake log data generator

# Usage

## Config file format

```yaml
---
# List of patterns to generate logs for. Currently logs are generated for all specified patterns evenly.
patterns:
  - '{"name":"{username}", "age":"{int}", "Moto": "{sentence}", "Summary": "{paragraph}", "ip_addr": "{ipv4}", "email": "{email}", "timestamp":"{ts_rfc3389}"}'

# Works for limiting the EPS to less than 2000. Cannot achieve more than ~10k eps with this setting on.
# Set to 0 for generating uncapped EPS
eps: 10
```

## Generate logs and output to STDOUT

```shell
fakelogs generate -c </path/to/config/file.yaml>
```

## Benchmark generating 100K logs with uncapped EPS

```shell
fakelogs benchmark -c </path/to/config/file.yaml>

Benchmarking ...
100000 log events generated in 1.781622306 seconds
EPS: 56129
```

## Benchmark using the EPS value specified in config file

```shell
fakelogs benchmark -c </path/to/config/file.yaml> --eps

Benchmarking ...
1500 log events generated in 1.161693726 seconds
EPS: 1291
```