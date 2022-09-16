FROM rust:latest as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev

RUN USER=root cargo new --bin app
WORKDIR /app
COPY ./ .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/fakelogs ./

ENTRYPOINT ["/app/fakelogs"]