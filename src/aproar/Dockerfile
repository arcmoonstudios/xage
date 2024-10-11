# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/aproar

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/aproar src/aproar

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/aproar/target/release/aproar /usr/local/bin/aproar

# Run the application
CMD ["aproar"]
