FROM rust:1.61

EXPOSE 8080

RUN apt update; apt install npm -y

WORKDIR /usr/src/friendly-football-bets

COPY ./ .

RUN cargo install --path ffb_cli/

WORKDIR /usr/src/friendly-football-bets/ffb_server

RUN bash install_styles.sh

RUN cargo b --release

CMD cargo r --bin ffb_server --release
