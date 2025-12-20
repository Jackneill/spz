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

assets:
	mkdir assets || true
	curl -o assets/racoonfamily.spz https://github.com/nianticlabs/spz/blob/main/samples/racoonfamily.spz
	curl -o assets/hornedlizard.spz https://github.com/nianticlabs/spz/blob/main/samples/hornedlizard.spz

clean:
	rm -rf ./target
