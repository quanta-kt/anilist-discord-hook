FROM rust:1.73.0 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim as runner
RUN apt-get update
RUN apt-get install -y libssl3 ca-certificates
COPY --from=builder /usr/local/cargo/bin/anilist-hook /usr/local/bin/anilist-hook

CMD [ "anilist-hook" ]
