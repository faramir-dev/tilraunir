FROM ubuntu:latest as builder
USER root
WORKDIR /
RUN set -eux; \
    apt update; \
    DEBIAN_FRONTEND=noninteractive apt install -y \
      bison \
      build-essential \
      clang \
      gcc \
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
      ripgrep;

RUN useradd -mU dev
WORKDIR /home/dev/
USER dev
