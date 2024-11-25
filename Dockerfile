FROM rust:alpine as builder
WORKDIR /usr/src/rustevalbot
COPY . .
RUN cargo install --path .

FROM alpine
COPY --from=builder /usr/local/cargo/bin/rustevalbot /usr/local/bin/rustevalbot
CMD ["rustevalbot"]
