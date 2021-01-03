# https://github.com/riscv/riscv-gnu-toolchain

FROM ubuntu:latest as builder
USER root
WORKDIR /
RUN set -eux; \
    apt update; \
    DEBIAN_FRONTEND=noninteractive apt install -y \
      autoconf \
      automake \
      autotools-dev \
      bc \
      bison \
      build-essential \
      clang \
      cmake \
      curl \
      flex \
      flex \
      g++-10 \
      gawk \
      git \
      gperf \
      libc-ares-dev \
      libexpat-dev \
      libgcrypt-dev \
      libgmp-dev \
      libmpc-dev \
      libmpfr-dev \
      libssl-dev \
      libtool \
      llvm \
      mc \
      neovim \
      patchutils \
      pkg-config \
      python3 \
      ripgrep \
      texinfo \
      zlib1g-dev \
      ;


RUN useradd -mU dev
WORKDIR /home/dev/

RUN set -eux; \
    mkdir /opt/riscv; \
    chown dev.dev /opt/riscv;

USER dev
RUN set -eux; \
    git clone --recursive https://github.com/riscv/riscv-gnu-toolchain; \
    cd riscv-gnu-toolchain; \
    git submodule update --init --recursive; \
    mkdir build; \
    cd build; \
    ../configure --prefix=/opt/riscv --enable-multilib; \
    make linux;
