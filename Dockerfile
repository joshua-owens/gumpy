FROM rust:latest
WORKDIR /usr/src/app
RUN cargo install diesel_cli --no-default-features --features postgres
COPY . .
EXPOSE 3000
CMD ["cargo", "run"]
