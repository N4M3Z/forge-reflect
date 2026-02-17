# forge-reflect — build, test, and lint

.PHONY: help build test lint check clean

help:
	@echo "forge-reflect targets:"
	@echo "  make build   Compile Rust binaries"
	@echo "  make test    Run all tests"
	@echo "  make lint    Clippy + fmt + shellcheck + semgrep"
	@echo "  make check   Verify module structure"
	@echo "  make clean   Remove build artifacts"

build:
	cargo build --release

test:
	cargo test
	@if [ -f tests/test.sh ]; then bash tests/test.sh; fi

lint:
	cargo fmt --check
	cargo clippy -- -D warnings
	@if find . -name '*.sh' -not -path '*/target/*' | grep -q .; then \
	  find . -name '*.sh' -not -path '*/target/*' | xargs shellcheck -S warning 2>/dev/null || true; \
	fi
	@if command -v semgrep >/dev/null 2>&1; then semgrep scan --config=p/owasp-top-ten --metrics=off --quiet . 2>/dev/null || true; fi

check:
	@test -f module.yaml && echo "  ok module.yaml" || echo "  MISSING module.yaml"
	@test -f Cargo.toml && echo "  ok Cargo.toml" || echo "  MISSING Cargo.toml"
	@test -d hooks && echo "  ok hooks/" || echo "  MISSING hooks/"

clean:
	cargo clean
