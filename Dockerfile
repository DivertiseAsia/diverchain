FROM rustlang/rust:nightly

RUN cargo install cargo-watch
RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /usr/src/app


CMD ["cargo", "run"]
