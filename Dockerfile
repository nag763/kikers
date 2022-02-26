FROM rust:1.57

WORKDIR /usr/src/friendly-football-bets

COPY ./ .

CMD cargo install --path .

RUN ffb
