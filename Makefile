.POSIX:

#
# # Makefile for Sylan
#
# This exists so that those not familiar with Rust tooling, such as Cargo, can
# easily build and test the project using the well-known `make` and `make
# verify` commands on POSIX systems.
#
# See README.md for more details.
#

CARGO=cargo
CARGO_FLAGS=
RUSTUP=rustup
RUSTUP_FLAGS=
RUST_CHANNEL=stable


all: install-toolchain-components verify build

verify: check-fmt check-clippy
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(CARGO) test $(CARGO_FLAGS)

clean:
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(CARGO) clean $(CARGO_FLAGS)


help:
	@echo The Sylan Programming Language
	@echo
	@echo "If you're familiar with Rust's tooling such as Cargo, you should use"
	@echo it directly rather than this Makefile.
	@echo
	@echo Targets:
	@echo "  all                          Build a production executable."
	@echo "  verify                       Check linting rules, formatting, and run tests."
	@echo "  clean                        Clean previously build artifacts."
	@echo
	@echo "  help                         View this help section."
	@echo "  install-toolchain            Install the Rust toolchain."
	@echo "  install-toolchain-components Install Rust toolchain components such as formatter and linter."
	@echo "  check-fmt                    Validate the source formatting."
	@echo "  fmt                          Format the source automatically."
	@echo "  check-clippy                 Lint the source."
	@echo "  build-dev                    Build a development executable."
	@echo "  build                        Build a production executable without verifications steps."

install-toolchain:
	$(RUSTUP) install $(RUST_CHANNEL)

install-toolchain-components: install-toolchain
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(RUSTUP) component add rustfmt clippy $(RUSTUP_FLAGS)

check-fmt:
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(CARGO) fmt -- --check $(CARGO_FLAGS)

fmt:
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(CARGO) fmt $(CARGO_FLAGS)

check-clippy:
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(CARGO) clippy --all-targets --all-features $(CARGO_FLAGS)

build-dev:
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(CARGO) build $(CARGO_FLAGS)

build:
	$(RUSTUP) run $(RUST_CHANNEL) $(RUSTUP_FLAGS) $(CARGO) build --release $(CARGO_FLAGS)

