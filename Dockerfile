FROM rust:1.61.0-buster as builder

WORKDIR /usr/src/newsfrwdr
COPY . .

RUN cargo install --path .

FROM debian:buster-slim

LABEL maintainer="rdk31 <rdk31@protonmail.com>"

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/newsfrwdr /usr/local/bin/newsfrwdr

CMD ["newsfrwdr", "-c", "/config/config.toml"]