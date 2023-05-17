FROM --platform=linux/amd64 rustlang/rust:nightly as builder

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update 
RUN apt-get install -y cmake clang
RUN cargo install cargo-fuzz

ADD . /Sunscreen/

WORKDIR /Sunscreen/fuzz/

RUN cargo +nightly fuzz build

FROM --platform=linux/amd64 rustlang/rust:nightly

COPY --from=builder /Sunscreen/fuzz/target/x86_64-unknown-linux-gnu/release/multiply /