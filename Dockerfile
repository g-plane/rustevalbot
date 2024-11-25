FROM rust:latest

WORKDIR /usr/src/rustevalbot
COPY . .

RUN cargo install --path .

CMD ["rustevalbot"]
