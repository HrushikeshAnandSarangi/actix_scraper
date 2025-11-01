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
RUN cargo build --release

# Runtime stage
FROM ubuntu:24.04

# Install OpenSSL, CA certificates, and Google Chrome
RUN apt-get update && \
    apt-get install -y openssl ca-certificates wget gnupg && \
    wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | gpg --dearmor -o /usr/share/keyrings/google-chrome-keyring.gpg && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/google-chrome-keyring.gpg] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list && \
    apt-get update && \
    apt-get install -y google-chrome-stable && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy the binary and static files from the builder stage
COPY --from=builder /app/target/release/actix_scraper /app/app
COPY --from=builder /app/static ./static

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Use an environment variable for the port.
ENV PORT=8000

# Expose the port defined by the $PORT environment variable
EXPOSE $PORT

# Run the binary
# We add --no-sandbox which is required for running Chrome as root or in a container
CMD ["./app", "--no-sandbox"]

