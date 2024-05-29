# BUILD TIME
FROM rust:latest as build

WORKDIR /app

COPY . .

RUN cargo build --release --features "strict"

# RUN TIME
FROM rust:latest as runtime
RUN groupadd -r git && \
    useradd -r -g git -d /git-home -s /sbin/nologin git-user
WORKDIR /app
RUN mkdir /github
RUN chown -R git-user:git /git-home
RUN chown -R git-user:git /github
USER git-user
RUN git config --global --add safe.directory '/git-home/app'
RUN git config --global --add safe.directory '/github'

COPY --from=build /app/target/release .

CMD ["/app/flexvers"]
