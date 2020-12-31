FROM ubuntu:latest as builder
USER root
WORKDIR /
RUN set -eux; \
    apt update; \
    DEBIAN_FRONTEND=noninteractive apt install -y \
      bison \
      build-essential \
      clang \
      g++-10 \
      curl \
      cmake \
      flex \
      git \
      libc-ares-dev \
      llvm \
      libgcrypt-dev \
      libgmp-dev \
      libmpfr-dev \
      libmpc-dev \
      libssl-dev \
      mc \
      neovim \
      pkg-config \
      ripgrep;

RUN useradd -mU dev
WORKDIR /home/dev/

RUN set -eux; \
    mkdir /opt/gcc-trunk; \
    chown dev.dev /opt/gcc-trunk

USER dev
RUN set -eux; \
    git clone git://gcc.gnu.org/git/gcc.git gcc-trunk; \
    cd gcc-trunk; \
    mkdir build; \
    cd build; \
    ../configure --prefix=/opt/gcc-trunk --disable-multilib; \
    make -j8; \
    make install;
