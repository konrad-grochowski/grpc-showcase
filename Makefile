# Variables
CARGO := cargo

# Targets
.PHONY: all build test clean precommit clippy_dirty_fix fmt e2e_test

all: build

build:
	$(CARGO) build

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt

clippy_dirty_fix:
	$(CARGO) clippy --fix --allow-dirty

precommit:  clippy-dirty-fix fmt test


clean:
	$(CARGO) clean

e2e_test:
	docker compose up --build -d | tee e2e_test.log;
	cargo test  --release   -- --ignored --nocapture


generate_certificates:
	./cert_gen.sh self-signed-certs/grpc-store grpc-store 0.0.0.0
	./cert_gen.sh self-signed-certs/rest-api rest-api 127.0.0.1
