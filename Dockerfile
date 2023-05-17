FROM --platform=$BUILDPLATFORM rust:1.69 as builder
# ARG CARGO_BUILD_TARGET=
ENV CARGO_TERM_COLOR=always \
  CARGO_NET_GIT_FETCH_WITH_CLI=true \
  CC_aarch64_unknown_linux_musl=clang \
  AR_aarch64_unknown_linux_musl=llvm-ar \
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"
WORKDIR /app

ARG TARGETPLATFORM
ARG BUILDPLATFORM
RUN echo "BUILDPLATFORM: $BUILDPLATFORM"
RUN echo "TARGETPLATFORM: $TARGETPLATFORM"
RUN case "$TARGETPLATFORM" in \
  "linux/amd64") \
  export CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu \
  ;; \
  "linux/arm64") \
  export CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu \
  ;; \
  esac; \
  echo "CARGO_BUILD_TARGET: $CARGO_BUILD_TARGET"

RUN echo "CARGO_BUILD_TARGET: $CARGO_BUILD_TARGET"

# RUN apt-get update && apt-get install -y musl-tools clang llvm libudev-dev
# RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

RUN apt update && apt upgrade -y
RUN apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross

RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
RUN rustup toolchain install stable-aarch64-unknown-linux-gnu

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
  CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

COPY . .

RUN case "$TARGETPLATFORM" in \
  "linux/amd64") \
  cargo build --release --target=x86_64-unknown-linux-gnu \
  ;; \
  "linux/arm64") \
  cargo build --release --target=aarch64-unknown-linux-gnu \
  ;; \
  esac;

FROM debian:bullseye-slim AS runtime
ARG CARGO_BUILD_TARGET=
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
WORKDIR /app
COPY --from=builder /app/target/${CARGO_BUILD_TARGET}/release/jarvis-homewizard-exporter .
ENTRYPOINT ["./jarvis-homewizard-exporter"]