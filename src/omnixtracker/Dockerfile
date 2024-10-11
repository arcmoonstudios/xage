# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/omnixtracker

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/omnixtracker src/omnixtracker

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/omnixtracker/target/release/omnixtracker /usr/local/bin/omnixtracker

# Run the application
CMD ["omnixtracker"]
