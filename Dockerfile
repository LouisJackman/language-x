
FROM rust:1.33.0-slim-stretch as builder

ARG RUST_CHANNEL=stable

ENV DEBIAN_FRONTEND noninteractive
ENV RUST_CHANNEL=$RUST_CHANNEL

RUN apt-get update --yes \
    && apt-get install make --yes --no-install-recommends \
    && rm -fr /var/lib/apt/lists/*

WORKDIR /opt/sylan
COPY . .

RUN make install-toolchain-components RUST_CHANNEL="$RUST_CHANNEL"

RUN ["chown", "-R", "nobody", "/opt/sylan"]
USER nobody

RUN make verify RUST_CHANNEL="$RUST_CHANNEL"
RUN make build RUST_CHANNEL="$RUST_CHANNEL"
RUN make build-dev RUST_CHANNEL="$RUST_CHANNEL"



FROM kcov/kcov:v36 as coverage

ENV DEBIAN_FRONTEND noninteractive

# Installing wget instead of curl to do the HTTP donwload in
# `install-coverage-tools.sh` means there is not a newly installed curl version
# to clash with the exact one that the preinstalled kcov binary expects.
RUN apt-get update --yes \
    && apt-get install wget --yes \
    && rm -fr /var/lib/apt/lists/*

COPY --from=builder /opt/sylan/target/debug /opt/debug
COPY ./scripts /opt/scripts

RUN ["sh", "/opt/scripts/install-coverage-tools.sh"]

ENTRYPOINT ["sh", "/opt/scripts/check-coverage.sh"]
CMD ["/opt/debug"]



FROM debian:buster-slim

COPY --from=builder /opt/sylan/target/release/sylan /opt/sylan

RUN ["chown", "nobody", "/opt/sylan"]
USER nobody

WORKDIR /opt
ENTRYPOINT ["/opt/sylan"]

