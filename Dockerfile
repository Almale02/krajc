FROM rust:1.77.0
WORKDIR /usr/src/rust_docker
COPY /. /.
RUN cargo run --release