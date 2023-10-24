# Use an existing lightweight base image (this one is based on Debian and has a minimal installation)
FROM rust:latest

## Install s4cmd
RUN apt-get update && apt-get install -y s4cmd


WORKDIR /app
COPY Cargo.toml .
COPY src/ src/
RUN cargo build --release

WORKDIR /
COPY init.sh .

ENTRYPOINT ["./init.sh"]