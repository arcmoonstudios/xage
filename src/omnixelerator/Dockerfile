# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/omnixelerator

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/omnixelerator src/omnixelerator

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/omnixelerator/target/release/omnixelerator /usr/local/bin/omnixelerator

# Run the application
CMD ["omnixelerator"]
