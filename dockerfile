FROM rust:slim AS builder
WORKDIR /app

COPY . .
RUN cargo build --bin citesync-server --release

FROM debian:trixie-slim

ENV CITESYNC_URL="http://127.0.0.1:3000"
ENV CITESYNC_DISABLE_DOCS=false
ENV CITESYNC_PORT="3000"
ENV CITESYNC_DATA_DIR="./data"
ENV CITESYNC_BIB_PATH="sources.json"

WORKDIR /app

COPY --from=builder /app/target/release/citesync-server ./citesync-server

EXPOSE 3000
CMD ["./citesync-server"]

