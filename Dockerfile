# ---- Build Stage ----
FROM rust:slim-bullseye AS builder

WORKDIR /app

# copy manifests
COPY Cargo.toml Cargo.lock ./

# copy real source
COPY src ./src

# copy templates
COPY templates ./templates

# build real binary
RUN cargo build --release

# ---- Runtime Stage ----
FROM debian:bullseye-slim

WORKDIR /app

COPY templates/static ./templates/static

COPY --from=builder /app/target/release/tinyexpenses .

EXPOSE 8080

CMD ["./tinyexpenses", "--bind", "0.0.0.0:8080", "--db", "accounts"]