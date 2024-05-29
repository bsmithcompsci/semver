# BUILD TIME
FROM rust:latest as build

WORKDIR /app

COPY . .

RUN cargo build --release --features "strict"

# RUN TIME
FROM rust:latest as runtime
RUN mkdir -p /github/workspace
RUN groupadd -r git && \
    useradd -r -g git -d /github -s /sbin/nologin git-user
WORKDIR /app
RUN chown -R git-user:git /github
USER git-user
RUN git config --global --add safe.directory '/github/workspace'

COPY --from=build /app/target/release .

CMD ["/app/flexvers"]
