ARG TARGET=x86_64-unknown-linux-musl
ARG BINARY_NAME=run-sh
ARG SQLX_OFFLINE=true

FROM --platform=$TARGETPLATFORM clux/muslrust:1.81.0-stable AS chef
ARG TARGET
ARG BINARY_NAME
ARG SQLX_OFFLINE
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target "$TARGET" --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY . .
RUN cargo build --release --target "$TARGET" --bin $BINARY_NAME

FROM --platform=$TARGETPLATFORM alpine:3.18.4 as runtime
WORKDIR /app
ARG TARGET
ARG BINARY_NAME
RUN apk --no-cache add ca-certificates

COPY --from=builder /app/target/$TARGET/release/$BINARY_NAME /app/bin

COPY languages languages

CMD /app/bin
