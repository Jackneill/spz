just := "just"
cargo := "cargo"
docker := "podman"

app_name := "spz"

export RUST_BACKTRACE := "full"
export DOCKER_BUILDKIT := "1"

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

flatpak-prepare: uv-install
	flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
	flatpak install -y org.freedesktop.Sdk.Extension.rust-stable/x86_64/25.08
	flatpak install -y org.flatpak.Builder

flatpak-lint: flatpak-cargo-sources
	flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest flatpak/org.jackneill.spz.yml
	rm flatpak/cargo-sources.json

flatpak-build: flatpak-cargo-sources
	flatpak run org.flatpak.Builder \
		--install \
		--install-deps-from=flathub \
		--force-clean \
		--user \
		-y \
		flatpak/repo \
		flatpak/org.jackneill.spz.yml
	rm flatpak/cargo-sources.json

flatpak-cargo-sources: uv-install
	uv run https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/refs/heads/master/cargo/flatpak-cargo-generator.py -o flatpak/cargo-sources.json Cargo.lock

uv-install:
	[ -x "$(command -v uv)" ] || curl -LsSf https://astral.sh/uv/install.sh | sh

docker-build-image:
	{{docker}} build -t {{app_name}} .

dr *args:
	{{docker}} run --rm -it -v "${PWD}:/app" -w /app {{app_name}} {{args}}
