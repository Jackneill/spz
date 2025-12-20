just := "just"
cargo := "cargo"

export RUST_BACKTRACE := "full"

default:
	{{just}} --list

test:
	{{cargo}} nextest run \
		-v \
		--all-features \
		--bins \
		--examples \
		--tests \
		--all-targets \
		-j num-cpus \
		--workspace
	#-- --nocapture

lint:
	{{cargo}} fmt --check
	{{cargo}} clippy
	{{cargo}} deny check

bench:
	{{cargo}} bench \
		--all-features \
		--benches \
		--profile release \
		--workspace

build-release: lint test
	{{cargo}} build --release

build:
	{{cargo}} build

run *args:
	{{cargo}} run --bin spz {{args}}

runr *args:
	{{cargo}} run --release --bin spz {{args}}

clean:
	rm -rf ./target
