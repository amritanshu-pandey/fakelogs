FROM rust:1.99-slim AS builder

RUN rustup target add x86_64-unknown-linux-musl \
    && apt-get update \
    && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build the dependencies on their own first, so this layer is cached and
# only changes when Cargo.toml or Cargo.lock do.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked --target x86_64-unknown-linux-musl \
    && rm -rf src

COPY src ./src
# Make sure cargo rebuilds main.rs rather than reusing the placeholder build.
RUN touch src/main.rs \
    && cargo build --release --locked --target x86_64-unknown-linux-musl

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/fakelogs ./
COPY example ./example

ENTRYPOINT ["/app/fakelogs"]
