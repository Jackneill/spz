just := "just"
cargo := "cargo"

default:
	{{just}} --list

test:
	RUST_BACKTRACE=full {{cargo}} test \
		-vv \
		--all-features \
		--bins \
		--examples \
		--tests \
		--benches \
		--all-targets \
		--workspace
	#-- --nocapture

lint:
	RUST_BACKTRACE=full {{cargo}} fmt --check
	RUST_BACKTRACE=full {{cargo}} clippy
	RUST_BACKTRACE=full {{cargo}} deny check

bench:
	RUST_BACKTRACE=full {{cargo}} bench \
		-vv \
		--profile release \
		--all-features \
		--bins \
		--examples \
		--tests \
		--benches \
		--all-targets \
		--workspace

build-release: lint test
	{{cargo}} build --release

build:
	{{cargo}} build

clean:
	rm -rf ./target
