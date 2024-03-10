FROM rust as builder

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl ca-certificates

WORKDIR /usr/app

COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/threads_crush /usr/app/threads_crush

EXPOSE 443

ENTRYPOINT ["/usr/app/threads_crush"]
CMD ["start", "-e", "production"]