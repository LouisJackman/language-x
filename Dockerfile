FROM rust:1.47.0-slim-buster as builder

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



FROM kcov/kcov:v38 as coverage

RUN apt-get update --yes \
    && apt-get install jq --yes --no-install-recommends \
    && rm -fr /var/lib/apt/lists/*

COPY --from=builder /opt/sylan/target/debug /opt/debug
COPY ./scripts /opt/scripts

RUN ["mkdir", "/opt/coverage-results"]
VOLUME /opt/coverage-results

ENTRYPOINT ["sh", "/opt/scripts/check-coverage.sh"]
CMD ["/opt/debug"]



FROM debian:buster-slim

COPY --from=builder /opt/sylan/target/release/sylan /usr/local/bin/sylan
RUN ["chmod", "ugo+x", "/usr/local/bin/sylan"]

RUN ["useradd", "-m", "user"]
USER user
WORKDIR /home/user

VOLUME /home/user
ENTRYPOINT ["/usr/local/bin/sylan"]

