FROM rust:1.48 as builder

RUN mkdir /src
WORKDIR /src
COPY . .
RUN cargo build --workspace --release

FROM debian:stable-slim

RUN apt-get update && apt-get -y install openssl

COPY --from=builder /src/target/release/progress-reporter /bin/
COPY --from=builder /src/target/release/prophecy-on-demand /bin/

RUN mkdir /messages
VOLUME /messages
WORKDIR /