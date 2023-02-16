FROM rust:1.61 AS builder
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder ./target/release/larkbot-notifier ./target/release/larkbot-notifier

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates  \
    netbase \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/ \
    && apt-get autoremove -y && apt-get autoclean -y

EXPOSE 3000

CMD ["/target/release/larkbot-notifier"]