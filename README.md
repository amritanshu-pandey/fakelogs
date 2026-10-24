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