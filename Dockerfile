FROM rust:1.76-slim

RUN apt-get update \
    && apt-get install -y -q \
        libssl-dev \
        pkg-config