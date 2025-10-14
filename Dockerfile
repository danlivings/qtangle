FROM rust:1.85 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN cargo build --release

FROM ubuntu:latest

WORKDIR /app
COPY --from=builder /app/target/release/qtangle /app/qtangle

CMD ["./qtangle"]