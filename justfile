# SPDX-License-Identifier: Apache-2.0 OR MIT

just := "just"
cargo := "cargo"
docker := "docker"

app_name := "spz"
c_binding_crate_name := "spz-binding-c"
container_img_name := "ghcr.io/jackneill/spz"

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
		--exclude spz-fuzz \
		-j num-cpus \
		--workspace
	#-- --nocapture

testp project:
	{{cargo}} nextest run \
		-v \
		--all-features \
		-p {{project}} \
		-j num-cpus

fuzz:
	#!/usr/bin/env bash
	set -euxo pipefail

	max_len=`calc 4096*32 | xargs`

	for f in `cargo fuzz list`; do \
		echo "Running fuzz target: ${f}"; \
		{{cargo}} +nightly fuzz run \
			--release \
			--all-features \
			--sanitizer none \
			$f -- -max_len=$max_len -max_total_time=60; \
	done

mutants:
	{{cargo}} mutants -d crates/spz

lint:
	{{cargo}} fmt --check
	{{cargo}} clippy
	uvx reuse lint

lint-ci:
	curl --data-binary @codecov.yml https://codecov.io/validate
	go install github.com/rhysd/actionlint/cmd/actionlint@latest
	actionlint

lint-py:
	uvx ruff check crates/spz-binding-python

lint-pyf:
	uvx ruff check --fix crates/spz-binding-python

bench: assets
	{{cargo}} bench \
		--all-features \
		--benches \
		--profile release \
		--workspace

bench-native: assets
	RUSTFLAGS='-C target-cpu=native' {{cargo}} bench \
		--all-features \
		--benches \
		--profile release \
		--workspace

security:
	{{cargo}} deny check
	{{cargo}} audit

build-release: lint test
	#RUSTFLAGS='-C target-cpu=native' {{cargo}} build --release
	{{cargo}} build --release

build-release-native: lint test
	RUSTFLAGS='-C target-cpu=native' {{cargo}} build --release

build:
	{{cargo}} build

build-native:
	RUSTFLAGS='-C target-cpu=native' {{cargo}} build

cinstall:
	{{cargo}} cinstall \
		--path crates/{{c_binding_crate_name}} \
		--destdir=./target/lib{{app_name}} \
		--prefix=/usr \
		--libdir=/usr/lib64

cbuild:
	{{cargo}} build -p {{c_binding_crate_name}}
	{{cargo}} cbuild --destdir=./target/lib{{app_name}} \
		--prefix=/usr \
		--libdir=/usr/lib64

cbuild-release:
	{{cargo}} cbuild --destdir=./target/lib{{app_name}} \
		--prefix=/usr \
		--release \
		--libdir=/usr/lib64

run *args:
	{{cargo}} run --bin spz {{args}}

runr *args:
	{{cargo}} run --release --bin spz {{args}}

assets:
	mkdir assets || true
	test -e assets/racoonfamily.spz || curl -L -o assets/racoonfamily.spz https://github.com/nianticlabs/spz/raw/refs/heads/main/samples/racoonfamily.spz
	test -e assets/hornedlizard.spz || curl -L -o assets/hornedlizard.spz https://github.com/nianticlabs/spz/raw/refs/heads/main/samples/hornedlizard.spz

flatpak-prepare:
	flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
	flatpak install -y org.freedesktop.Sdk.Extension.rust-stable/x86_64/25.08
	flatpak install -y org.flatpak.Builder

flatpak-lint:
	flatpak run --command=flatpak-builder-lint \
		org.flatpak.Builder \
		manifest flatpak/io.github.jackneill.spz.yml

flatpak-build:
	rm -rf ./.flatpak-build
	rm -rf ./flatpak/repo
	rm -rf ./flatpak/build

	flatpak run org.flatpak.Builder \
		-v \
		--install \
		--install-deps-from=flathub \
		--force-clean \
		--disable-rofiles-fuse \
		--repo=flatpak/repo \
		--user \
		-y \
		flatpak/build \
		flatpak/io.github.jackneill.spz.yml

	flatpak run --command=flatpak-builder-lint -v org.flatpak.Builder repo flatpak/repo

uv-install:
	[ -x "$(command -v uv)" ] || curl -LsSf https://astral.sh/uv/install.sh | sh

docker-build-image:
	{{docker}} build -t {{container_img_name}} .

docker-build-image-multi-arch:
	{{docker}} buildx build \
		--platform linux/amd64,linux/arm64,linux/arm/v6,linux/riscv64 \
		-t {{container_img_name}} .

docker-run *args:
	{{docker}} run --rm -it -v "${PWD}:/app" -w /app {{container_img_name}} {{args}}

py:
	#!/usr/bin/env bash
	set -euxo pipefail

	pyenv="crates/spz-binding-python/.venv"

	pushd .
	cd crates/spz-binding-python && uv venv
	popd

	source "${pyenv}/bin/activate"

	uv pip install pytest numpy maturin
	#uv tool install maturin

	uv run -p "${pyenv}" \
		maturin develop --uv \
		--manifest-path crates/spz-binding-python/Cargo.toml \
		--compression-method zstd
	uv run -p "${pyenv}" python -i crates/spz-binding-python/dev/shell_prefill.py

	deactivate

py-test:
	#!/usr/bin/env bash
	set -euxo pipefail

	pyenv="crates/spz-binding-python/.venv"

	uv pip install -p "${pyenv}" pytest numpy
	uv run -p "${pyenv}" python -m pytest crates/spz-binding-python/pypkg/tests/

py-build:
	uvx maturin build --release --manifest-path crates/spz-binding-python/Cargo.toml

py-publish:
	uvx maturin publish --manifest-path crates/spz-binding-python/Cargo.toml

shellcheck script:
	{{docker}} run --rm -v "${PWD}:/mnt" koalaman/shellcheck:stable {{script}}

clean:
	rm -rf ./target
	rm -rf ./flatpak/repo
	rm -rf ./flatpak/build
	rm -rf ./.flatpak-builder
	rm -rf ./.ruff_cache
	# remove image locally
	{{docker}} rmi {{container_img_name}}:latest
