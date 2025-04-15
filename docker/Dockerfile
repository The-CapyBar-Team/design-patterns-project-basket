FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY ../ .

RUN apt update                                                          && \
    apt install -y protobuf-compiler                                    && \
    cargo build --release --bin basket_balancer --bin basket_service

CMD ["cargo", "run", "--release", "--bin", "basket_service"]
