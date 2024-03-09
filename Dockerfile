FROM rust:bookworm as builder

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/app

COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/threads_crush /usr/app/threads_crush

ENTRYPOINT ["/usr/app/threads_crush start -e production"]