FROM rust:alpine3.16 as builder
RUN apk add --no-cache build-base libressl-dev

RUN USER=root cargo new --bin alligator
WORKDIR /alligator

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/alligator*
RUN cargo build --release && strip ./target/release/alligator

FROM alpine:3.16
RUN apk --no-cache add libressl

COPY --from=builder /alligator/target/release/alligator /usr/local/bin/alligator
CMD ["/usr/local/bin/alligator"]
