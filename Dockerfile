FROM rust:1.47.0-slim-buster as rust-base

ARG RUST_CHANNEL=stable

ENV RUST_CHANNEL=$RUST_CHANNEL

RUN DEBIAN_FRONTEND=noninteractive apt-get update --yes \
    && DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
    make \
    && rm -fr /var/lib/apt/lists/*

WORKDIR /opt/sylan
RUN ["chown", "-R", "nobody:nogroup", "/opt/sylan"]

USER nobody

COPY --chown=nobody:nogroup . .

RUN make install-toolchain-components RUST_CHANNEL="$RUST_CHANNEL"


FROM rust-base as build

CMD make verify RUST_CHANNEL="$RUST_CHANNEL" \
    && make build RUST_CHANNEL="$RUST_CHANNEL"


FROM rust-base as coverage

USER root

RUN apt-get update --yes \
    && apt-get install --yes --no-install-recommends \
    g++ \
    libssl-dev \
    pkg-config \
    python3 \
    sudo \
    unzip \
    wget \
    && rm -fr /var/lib/apt/lists/*

RUN sed -i.bk \
    's/ALL$/NOPASSWD: ALL/' \
    /etc/sudoers

RUN ["usermod", "-a", "-G", "sudo", "nobody"]

USER nobody

RUN ["cargo", "install", "cargo-make"]

CMD ["cargo", "make", "coverage-tarpaulin"]



FROM debian:buster-20201012-slim

COPY --from=builder /opt/sylan/target/release/sylan /usr/local/bin/sylan
RUN ["chmod", "ugo+x", "/usr/local/bin/sylan"]

RUN ["useradd", "-m", "user"]
USER user
WORKDIR /home/user

VOLUME /home/user
ENTRYPOINT ["/usr/local/bin/sylan"]

