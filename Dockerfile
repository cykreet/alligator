FROM rust:alpine3.16 as build

WORKDIR /usr/src/alligator
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN cargo build --release
RUN rm src/*.rs

FROM rust:alpine3.16

COPY --from=build /usr/local/cargo/bin/alligator /usr/local/bin/alligator
ENTRYPOINT [ "/usr/local/bin/alligator" ]
