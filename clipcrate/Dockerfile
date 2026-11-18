# Keep the compiler aligned with rust-toolchain.toml and backend CI.
FROM rust:1.98.1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --locked --release -p clipcrate

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/clipcrate /usr/local/bin/clipcrate
ENV PORT=8080
EXPOSE 8080
CMD ["clipcrate"]
