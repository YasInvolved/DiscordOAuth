FROM rust:1.97-alpine AS builder

RUN apk add --no-cache musl-dev gcc make pkgconf openssl-dev openssl-libs-static

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/
COPY migration/src ./migration/src

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

RUN mkdir -p /app/data

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/discordoauthapi /app/discordoauthapi
EXPOSE 3000

ENV DATABASE_URL="sqlite:///app/data/auth.db?mode=rwc"
CMD ["/app/discordoauthapi"]