# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/model_interpretability

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/model_interpretability src/model_interpretability

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/model_interpretability/target/release/model_interpretability /usr/local/bin/model_interpretability

# Run the application
CMD ["model_interpretability"]
