FROM rust:1.92 AS builder
WORKDIR /usr/src/server
COPY . .
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /usr/src/server/target/x86_64-unknown-linux-musl/release/server /server
EXPOSE 3000
ENTRYPOINT ["/server"]
