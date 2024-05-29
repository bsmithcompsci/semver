# BUILD TIME
FROM rust:latest as build

WORKDIR /app

COPY . .

RUN cargo build --release --features "strict"

# RUN TIME
FROM rust:latest as runtime

WORKDIR /app

COPY --from=build /app/target/release .

CMD ["/app/flexvers"]
