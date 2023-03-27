# Set the base image to the official Rust image from Docker Hub
FROM javiani/rustymachine:latest

# Create a new directory to hold the Rust code
WORKDIR /app

# Clone the Rust code from the GitHub repository
RUN git clone https://github.com/aviani-sb/deno-proxy.git

# Compile and run the code
WORKDIR /app/deno-proxy

RUN cargo build --release

CMD ["/app/deno-proxy/target/release/deno-proxy"]


