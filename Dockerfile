FROM rust:1.95 AS builder
ARG TARGETARCH
WORKDIR /src
RUN case "$TARGETARCH" in \
    arm64) echo aarch64-unknown-linux-musl > /target ;; \
    amd64) echo x86_64-unknown-linux-musl  > /target ;; \
    *)     echo "Unsupported arch: $TARGETARCH" && exit 1 ;; \
    esac
RUN rustup target add "$(cat /target)"
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,id=$TARGETARCH,target=/src/target \
    export TARGET="$(cat /target)" \
    && cargo build --release --target "$TARGET" \
    && cp "target/$TARGET/release/byo-redis-server" /byo-redis-server

FROM scratch
WORKDIR /
COPY --from=builder /byo-redis-server .
EXPOSE 6379
CMD ["/byo-redis-server", "--bind", "0.0.0.0"]
