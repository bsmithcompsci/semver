FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build --release --features "strict"

CMD ["./target/release/flexvers"]
