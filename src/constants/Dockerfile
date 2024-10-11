# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/constants

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/constants src/constants

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/constants/target/release/constants /usr/local/bin/constants

# Run the application
CMD ["constants"]
