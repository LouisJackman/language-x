FROM rust:1.32-slim as builder

RUN ["rustup", "component", "add", "rustfmt", "clippy"]

WORKDIR /opt/sylan
COPY . .

RUN ["chown", "-R", "nobody", "/opt/sylan"]
USER nobody

RUN ["cargo", "fmt", "--", "--check"]
RUN ["cargo", "clippy", "--all-targets", "--all-features"]

RUN ["cargo", "build", "--release"]

FROM debian:buster-slim

COPY --from=builder /opt/sylan/target/release/sylan /opt/sylan

RUN ["chown", "nobody", "/opt/sylan"]
USER nobody

WORKDIR /opt
ENTRYPOINT ["/opt/sylan"]

