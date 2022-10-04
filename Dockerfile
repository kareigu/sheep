FROM rust as rust_builder

WORKDIR /usr/src

RUN USER=root cargo new --bin sheep

WORKDIR /usr/src/sheep

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release
RUN rm ./target/release/deps/sheep*
RUN rm src/*.rs

COPY ./src ./src
RUN cargo build --release

FROM debian:buster-slim

WORKDIR /usr/src/sheep


COPY --from=rust_builder /usr/src/sheep/target/release/sheep ./sheep-bot

CMD ./sheep-bot