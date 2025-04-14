FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY ../ .

RUN cargo build --release --bin basket_service

CMD ["cargo", "run", "--release", "--bin", "basket_service"]
