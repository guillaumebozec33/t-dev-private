FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig gcc libc-dev curl

WORKDIR /app
COPY Cargo.toml ./
COPY Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN touch src/main.rs
RUN cargo build --release

FROM alpine:3.19 AS runtime
RUN apk add --no-cache libgcc openssl ca-certificates
COPY --from=builder /app/target/release/rtc-backend /usr/local/bin/
EXPOSE 3001
CMD ["rtc-backend"]

# Tests
FROM builder AS tester
RUN rustup component add llvm-tools-preview
RUN CARGO_HOME=/usr/local/cargo cargo install cargo-llvm-cov --root /usr/local
RUN mkdir -p /app/coverage
CMD ["cargo", "llvm-cov", "--html", "--output-dir", "/app/coverage"]