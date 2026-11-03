# AX-1 testbed image (cgap courts/ax-testbed). Two-stage, digest-pinned.
# Pin provenance (witnessed 2026-11-02, lane AX-1):
#   AX commit: git ls-remote https://github.com/google/ax HEAD
#     -> b82f57ee20f8882bb62762fc8f0c24f1c3c27f86 (refreeze via same command).
#   Base digests: docker buildx imagetools inspect (this session).
ARG AX_REPO=https://github.com/google/ax
ARG AX_COMMIT=b82f57ee20f8882bb62762fc8f0c24f1c3c27f86
FROM golang:1.23-bookworm@sha256:167053a2bb901972bf2c1611f8f52c44d5fe7e762e5cab213708d82c421614db AS ax_builder
ARG AX_REPO
ARG AX_COMMIT
RUN git clone ${AX_REPO} /src/ax \
 && cd /src/ax && git switch --detach ${AX_COMMIT} \
 && git rev-parse HEAD > /ax_commit.txt
WORKDIR /src/ax
ENV CGO_ENABLED=0 GOFLAGS=-mod=mod GOTOOLCHAIN=auto
# DIVERGENCE (AX-1, 2026-11-02): go.mod at pin b82f57e requires go 1.27.1;
# golang:1.23 cannot load it directly. GOTOOLCHAIN=auto bootstraps 1.27.1.
# Stderr is NOT suppressed here — a module-load failure must be loud.
RUN go list ./cmd/ax-task-runner > /dev/null && \
      go build -o /out/ax-task-runner ./cmd/ax-task-runner
FROM debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3
LABEL org.opencontainers.image.title="cgap-ax-testbed"
LABEL org.opencontainers.image.description="AX-1: pinned runner + Go 1.23.4 + rust nightly + riscv32imc"
COPY --from=ax_builder /out/ax-task-runner /usr/local/bin/ax-task-runner
COPY --from=ax_builder /ax_commit.txt /usr/local/share/ax_commit.txt
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/ax-task-runner
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl git build-essential clang llvm python3 jq \
 && rm -rf /var/lib/apt/lists/*
ENV GOROOT=/usr/local/go CARGO_HOME=/usr/local/cargo RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/go/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin
ENV CGO_ENABLED=1
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y \
      --default-toolchain nightly \
      --profile minimal \
      --component llvm-tools-preview,rust-src \
 && /usr/local/cargo/bin/rustup target add riscv32imc-unknown-none-elf
# DOD-13 fix (TC-14 W6-2): the go tarball arch was hardcoded to amd64 while
# the bases are pulled arm64 on this host — amd64 go emitted `-m64` into the
# aarch64 gcc and every cgo build died. Derive the arch from TARGETARCH
# (set by BuildKit for every build, native or cross) and refuse loudly on
# anything outside the supported pair.
ARG TARGETARCH
RUN set -eu; case "$TARGETARCH" in \
      amd64) GO_TARBALL=go1.23.4.linux-amd64.tar.gz ;; \
      arm64) GO_TARBALL=go1.23.4.linux-arm64.tar.gz ;; \
      *) echo "DOD-13 FAIL: unsupported TARGETARCH '$TARGETARCH'" >&2; exit 1 ;; esac; \
    curl -sSfL "https://go.dev/dl/$GO_TARBALL" -o /tmp/go.tgz \
 && tar -C /usr/local -xzf /tmp/go.tgz && rm /tmp/go.tgz \
 && /usr/local/go/bin/go version
