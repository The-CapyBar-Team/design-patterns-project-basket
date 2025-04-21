FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY ../ .

ARG BASKET_PROTO_DIRECTORY_PATH
ENV BASKET_PROTO_DIRECTORY_PATH=${BASKET_PROTO_DIRECTORY_PATH}

RUN apt update                                                          && \
    apt install -y protobuf-compiler                                    && \
    cargo build --release --bin basket_request_generator --bin basket_service

CMD sh -c "cargo run --release --bin $BASKET_BIN"
