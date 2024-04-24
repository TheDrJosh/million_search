FROM rust:1.77.2
WORKDIR /usr/src/million
COPY ./million_backend ./million_backend
COPY ./proto ./proto
RUN cargo install --path ./million_backend
CMD ["million_backend"]