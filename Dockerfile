# Build image
# Necessary dependencies to build Mokuroku
FROM rust:slim-bullseye as build

RUN apt-get update && apt-get install -y \
  build-essential \
  autoconf \
  automake \
  pkg-config \
  libssl-dev \
  libtool

WORKDIR "/mokuroku"

# Cache cargo build dependencies by creating a dummy source
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.toml ./
RUN cargo build --release

COPY . .
RUN cargo build --release

# Release image
# Necessary dependencies to run Mokuroku
FROM debian:bullseye-slim

COPY --from=build /mokuroku/target/release/mokuroku .

CMD ["./mokuroku"]
