# FROM rust:1.77.2
# WORKDIR /usr/src/million
# RUN apt-get update
# RUN apt-get install -y protobuf-compiler
# COPY ./proto ./proto
# COPY ./million_backend ./million_backend
# RUN cargo install --path ./million_backend
# CMD ["million_backend"]

# Step 1: Compute a recipe file
FROM rust as planner
WORKDIR /usr/src/million
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Step 2: Cache project dependencies
FROM rust as cacher
WORKDIR /usr/src/million
RUN cargo install cargo-chef
COPY --from=planner /usr/src/million/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Step 3: Build the binary
FROM rust as builder
WORKDIR /usr/src/million
RUN apt-get update
RUN apt-get install -y protobuf-compiler
COPY . .
# Copy over the cached dependencies from above
COPY --from=cacher /usr/src/million/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin million_backend

# Step 4:
# Create a tiny output image.
# It only contains our final binary.
FROM rust as runtime
WORKDIR /usr/src/million
COPY --from=builder /usr/src/million/target/release/million_backend /usr/local/bin
ENTRYPOINT ["/usr/local/bin/million_backend"]