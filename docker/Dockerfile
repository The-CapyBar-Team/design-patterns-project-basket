FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY ../ .

ARG BASKET_PROTO_DIRECTORY_PATH

RUN apt update                                                          && \
    apt install -y protobuf-compiler                                    && \
    cargo build --release --bin basket_request_generator --bin basket_service

CMD ["cargo", "run", "--release", "--bin", "basket_service"]
