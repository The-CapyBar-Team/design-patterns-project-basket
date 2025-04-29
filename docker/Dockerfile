FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY ../ .

ARG MAX_BASKETS_COUNT
ARG BASKET_PROTO_DIRECTORY_PATH
ENV BASKET_PROTO_DIRECTORY_PATH=${BASKET_PROTO_DIRECTORY_PATH}

RUN apt update                                                          && \
    apt install -y protobuf-compiler                                    && \
    RUSTFLAGS="-Awarnings" cargo build --release --bin basket_request_generator --bin basket_service

CMD sh -c 'RUSTFLAGS="-Awarnings" cargo run --release --bin $BASKET_BIN'
