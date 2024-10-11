# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/data_processing

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/data_processing src/data_processing

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/data_processing/target/release/data_processing /usr/local/bin/data_processing

# Run the application
CMD ["data_processing"]
