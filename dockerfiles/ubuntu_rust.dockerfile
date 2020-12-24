FROM ubuntu:latest as builder
USER root
WORKDIR /
RUN apt update && DEBIAN_FRONTEND=noninteractive apt install -y \
    bison \
    build-essential \
    clang \
    curl \
    cmake \
    flex \
    git \
    libc-ares-dev \
    llvm \
    libgcrypt-dev \
    libssl-dev \
    mc \
    neovim \
    pkg-config \
    ripgrep

RUN useradd -mU dev
WORKDIR /home/dev/

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH
RUN set -eux; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs >.rustup-init.sh; \
    sh .rustup-init.sh -y --no-modify-path; \
    rm -v .rustup-init.sh; \
    cargo install bindgen; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME;

USER dev
