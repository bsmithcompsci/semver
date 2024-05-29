# BUILD TIME
FROM rust:latest as build

WORKDIR /build

COPY . .

RUN cargo build --release --features "strict"

# RUN TIME
FROM rust:latest as runtime
WORKDIR /app
RUN git config --global --add safe.directory '*'

COPY --from=build /build/target/release .
COPY --from=build /build/entrypoint.sh ./entrypoint.sh

CMD ["/app/flexvers"]
