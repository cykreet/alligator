FROM rust:alpine3.16 as builder
RUN apk add -q --update-cache --no-cache build-base openssl-dev

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

COPY --from=builder /alligator/target/release/alligator /bin/alligator
CMD ["/bin/alligator"]
