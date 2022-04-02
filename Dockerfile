FROM rust:1.57

EXPOSE 8080

RUN apt update; apt install npm -y

WORKDIR /usr/src/friendly-football-bets

COPY ./ .

RUN cargo install --path ffb-cli/

WORKDIR /usr/src/friendly-football-bets/ffb-server

RUN bash install_styles.sh

RUN cargo b --release

CMD cargo r --bin ffb-server --release
