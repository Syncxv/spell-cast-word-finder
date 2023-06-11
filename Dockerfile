# Select the rust base image
FROM rust:1.67 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin scf
WORKDIR /scf

# Copy your manifests into the new project
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./wordlist.txt ./wordlist.txt

# Copy your source tree
COPY ./src ./src

# Build for release.
# Offical rust images include rustc, cargo, rustup, with common rust build tools 
RUN cargo build --release

# Our Second stage, that will contain only the compiled binary
FROM ubuntu:latest

# Use static linking for portability
RUN apt-get update && apt-get install -y ca-certificates tzdata && rm -rf /var/lib/apt/lists/*

# copy the build artifact from the build stage
COPY --from=builder /scf/target/release/scf .
COPY --from=builder /scf/wordlist.txt .

EXPOSE 3030

# set the startup command to run your binary
CMD ["./scf"]
