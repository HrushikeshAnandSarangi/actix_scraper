# Build stage
FROM rustlang/rust:nightly AS builder

WORKDIR /app

# Copy manifests and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code and static files
COPY . .

# Build for release
# Clear out the dummy main.rs from before to avoid conflicts
RUN rm -f src/main.rs
RUN cargo build --release

# Runtime stage
FROM ubuntu:24.04

# Install OpenSSL and CA certificates
RUN apt-get update && \
    apt-get install -y openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy the *release* binary and static files from the builder stage
COPY --from=builder /app/target/release/actix_scraper /app/app
COPY --from=builder /app/static ./static

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# --- Modifications Start ---

# Use an environment variable for the port.
# This sets a default value of 8000.
ENV PORT=8000

# Expose the port defined by the $PORT environment variable
EXPOSE $PORT

# --- Modifications End ---

# Run the binary
# Your application code (e.g., in main.rs) should be written
# to read the "PORT" environment variable to know which port to bind to.
CMD ["./app"]

