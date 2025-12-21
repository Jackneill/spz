just := "just"
cargo := "cargo"

export RUST_BACKTRACE := "full"

default:
	{{just}} --list

test: assets
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

bench: assets
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

assets:
	mkdir assets || true
	test -e assets/racoonfamily.spz || curl -L -o assets/racoonfamily.spz https://github.com/nianticlabs/spz/raw/refs/heads/main/samples/racoonfamily.spz
	test -e assets/hornedlizard.spz || curl -L -o assets/hornedlizard.spz https://github.com/nianticlabs/spz/raw/refs/heads/main/samples/hornedlizard.spz

clean:
	rm -rf ./target
