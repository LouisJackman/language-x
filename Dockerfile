#
# # Sylan Dockerfile
#
# The build stages are split into three groups:
# * Base images: rebuilt occasionally, uploaded to a trusted container
#   registry, and reused by the CI constantly.
# * Building images: the steps to build Sylan from scratch, as Docker
#   image-building and container-running steps.
# * Main image: the actual image to run Sylan itself in an isolated container,
#   with no dependencies on previous stages except to COPY their produced
#   artifacts.
#

#
# ## Base Images
#


FROM rust:1.47.0-slim-buster as sylan-rust-base

ARG RUST_CHANNEL=stable

ENV RUST_CHANNEL=$RUST_CHANNEL

RUN DEBIAN_FRONTEND=noninteractive apt-get update --yes \
    && DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
    make \
    && rm -fr /var/lib/apt/lists/*

WORKDIR /opt/sylan
RUN ["chown", "-R", "nobody:nogroup", "/opt/sylan"]

USER nobody

RUN rustup install "$RUST_CHANNEL" \
    && rustup run "$RUST_CHANNEL" rustup component add rustfmt clippy


FROM sylan-rust-base as sylan-coverage-base

USER root

RUN DEBIAN_FRONTEND=noninteractive apt-get update --yes \
    && DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
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

# HACK: ignore failures on this command. It should fail because there isn't a
# project, but it should have automatically initialised by then, baking its
# initalisation into the image.
RUN cargo make coverage-tarpaulin || true


#
# ## Building Images
#


FROM sylan-rust-base as sylan-build

COPY --chown=nobody:nogroup . .

CMD make verify RUST_CHANNEL="$RUST_CHANNEL" \
    && make build RUST_CHANNEL="$RUST_CHANNEL"


FROM sylan-coverage-base as sylan-coverage

CMD ["cargo", "make", "coverage-tarpaulin"]


#
# ## Main Image
#


FROM debian:buster-20201012-slim

COPY --from=sylan-build /opt/sylan/target/release/sylan /usr/local/bin/sylan
RUN ["chmod", "ugo+x", "/usr/local/bin/sylan"]

RUN ["useradd", "-m", "user"]
USER user
WORKDIR /home/user

VOLUME /home/user
ENTRYPOINT ["/usr/local/bin/sylan"]
