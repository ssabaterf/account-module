# Use the official Rust image as the base image
FROM rust:1.68 as builder

# Create a new directory for our application
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Copy the source code to the container
COPY src/ ./src/

# Build the application with the optimizations enabled
RUN cargo build --release

# Use the Distroless image as the base image
FROM gcr.io/distroless/cc-debian10

# Set the working directory to the root of the file system
WORKDIR /app

# Copy the binary from the builder container to the final image
COPY --from=builder /app/target/release/account-module .

# Start the application
CMD ["/account-module"]
