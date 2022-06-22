FROM rust:latest AS builder

RUN update-ca-certificates

ENV USER=indexer
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app

COPY ./ .

RUN cargo build --release

FROM debian:buster-slim


COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

COPY --from=builder /app/target/release/indexer ./


USER indexer:indexer

CMD ["/app/indexer"]