FROM rust:latest
WORKDIR /usr/src/app
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
COPY . .
EXPOSE 3000
CMD ["cargo", "run"]
