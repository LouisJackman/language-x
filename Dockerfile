#
# ## Building Images
#
# the steps to build Sylan from scratch, as Docker
# image-building and container-running steps.
#


FROM registry.gitlab.com/sylan-language/sylan/rust-base as sylan-build

COPY --chown=nobody:nogroup . .

CMD make verify RUST_CHANNEL="$RUST_CHANNEL" \
    && make build RUST_CHANNEL="$RUST_CHANNEL"


FROM registry.gitlab.com/sylan-language/sylan/coverage-base as sylan-coverage

CMD ["cargo", "install", "cargo-tarpaulin"]
CMD ["cargo", "tarpaulin", "-v", "--all", "--out", "Xml", "--", "--test-threads", "2"]

#
# ## Main Image
#
# The actual image to run Sylan itself in an isolated container,
# with no dependencies on previous stages except to COPY their produced
# artifacts.
#


FROM debian:buster-20201012-slim

COPY --from=sylan-build /opt/sylan/target/release/sylan /usr/local/bin/sylan
RUN ["chmod", "ugo+x", "/usr/local/bin/sylan"]

RUN ["useradd", "-m", "user"]
USER user
WORKDIR /home/user

VOLUME /home/user
ENTRYPOINT ["/usr/local/bin/sylan"]
