# Stage 1: builder
FROM rust:1.86-bookworm AS builder

RUN apt-get update && apt-get install -y \
    cmake \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libasound2-dev \
    libx11-dev \
    libxtst-dev \
    libxi-dev \
    libleptonica-dev \
    tesseract-ocr \
    libtesseract-dev \
    libv4l-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release -p lifelog-server

# Stage 2: runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    tesseract-ocr \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/lifelog-server /usr/local/bin/lifelog-server

EXPOSE 7182

ENTRYPOINT ["lifelog-server", "serve"]
