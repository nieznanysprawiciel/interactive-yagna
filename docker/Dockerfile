FROM rust:1.48 as builder

RUN mkdir /src
WORKDIR /src
COPY . .
RUN cargo build --release -p progress-reporter

FROM debian:stable-slim

COPY --from=builder /src/target/release/progress-reporter /bin/
COPY --from=builder /src/target/release/prophecy-on-demand /bin/

VOLUME /messages
WORKDIR /