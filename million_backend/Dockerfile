FROM rust:1.77.2
WORKDIR /usr/src/million
RUN apt-get update
RUN apt-get install -y protobuf-compiler
COPY ./proto ./proto
COPY ./million_backend ./million_backend
RUN cargo install --path ./million_backend
CMD ["million_backend"]