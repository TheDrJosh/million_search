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
RUN apt-get update && \
    apt-get install -y nodejs \
    npm
COPY --from=planner /usr/src/million/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY --from=planner /usr/src/million/million_client/package.json package.json
COPY --from=planner /usr/src/million/million_client/package-lock.json package-lock.json
RUN npm i

# Step 3: Build the binary
FROM rust as builder
WORKDIR /usr/src/million
RUN apt-get update && \
    apt-get install -y nodejs \
    npm protobuf-compiler
COPY . .
# Copy over the cached dependencies from above
COPY --from=cacher /usr/src/million/node_modules million_client/node_modules
COPY --from=cacher /usr/src/million/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
WORKDIR /usr/src/million/million_client
RUN npm run build
RUN cargo build --release

# Step 4:
# Create a tiny output image.
# It only contains our final binary.
FROM rust as runtime
WORKDIR /usr/src/million
COPY --from=builder /usr/src/million/million_client/public /usr/src/million/public
COPY --from=builder /usr/src/million/target/release/million_client /usr/local/bin
ENTRYPOINT ["/usr/local/bin/million_client"]