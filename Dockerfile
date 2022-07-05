FROM rust:alpine3.16 as builder
RUN apk add build-base libressl-dev

WORKDIR /usr/src/alligator
COPY . .
RUN cargo build --release && strip target/release/alligator

FROM alpine:3.16
RUN apk add libressl

COPY --from=builder /usr/src/alligator/target/release/alligator /usr/local/bin/alligator
CMD ["/usr/local/bin/alligator"]
