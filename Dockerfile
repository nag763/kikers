FROM rust:1.57

WORKDIR /usr/src/friendly-football-bets

COPY ./ .

RUN cargo install sea-orm-cli --features sqlx-mysql

RUN sea-orm-cli migrate init

RUN sea-orm-cli migrate up

RUN cargo run
