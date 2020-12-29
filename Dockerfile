ARG rust_version
FROM rust:${rust_version}-buster as buildenv
MAINTAINER Diego Veralli "diego@diegoveralli.com"

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update

RUN apt-get install -y --no-install-recommends libdbus-1-dev

COPY . /source

WORKDIR /source

RUN cargo build --release

FROM scratch AS export-stage

COPY --from=buildenv /source/target/release/steam-auto-gamemode .
