# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/multi_modal

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/multi_modal src/multi_modal

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/multi_modal/target/release/multi_modal /usr/local/bin/multi_modal

# Run the application
CMD ["multi_modal"]
