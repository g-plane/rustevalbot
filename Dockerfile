FROM rust:latest AS builder
WORKDIR /usr/src/rustevalbot
COPY . .
RUN cargo install --path .

FROM debian:12-slim
COPY --from=builder /usr/local/cargo/bin/rustevalbot /usr/local/bin/rustevalbot
CMD ["rustevalbot"]
