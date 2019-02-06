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


all: install-compiler-components fmt check-clippy
	$(CARGO) build --release $(CARGO_FLAGS)

verify: check-fmt check-clippy
	$(CARGO) test $(CARGO_FLAGS)

clean:
	$(CARGO) clean $(CARGO_FLAGS)


help:
	@echo The Sylan Programming Language
	@echo
	@echo "If you're familiar with Rust's tooling, such as Cargo, you should use"
	@echo it directly rather than using this Makefile.
	@echo
	@echo Targets:
	@echo "  all                         Build a production executable."
	@echo "  verify                      Check linting rules, formatting, and run tests."
	@echo "  clean                       Clean previously build artifacts."
	@echo
	@echo "  help                        View this help section."
	@echo "  install-compiler-components Install Rust compiler components such as formatter and linter."
	@echo "  check-fmt                   Validate the source formatting."
	@echo "  fmt                         Format the source automatically."
	@echo "  check-clippy                Lint the source."
	@echo "  build-dev                   Build a development executable."

install-compiler-components:
	$(RUSTUP) component add rustfmt clippy $(RUSTUP_FLAGS)

check-fmt:
	$(CARGO) fmt -- --check $(CARGO_FLAGS)

fmt:
	$(CARGO) fmt $(CARGO_FLAGS)

check-clippy:
	$(CARGO) clippy --all-targets --all-features $(CARGO_FLAGS)

build-dev:
	$(CARGO) build $(CARGO_FLAGS)

