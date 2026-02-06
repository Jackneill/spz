#!/usr/bin/env bash

set -euxo pipefail

DEBIAN_FRONTEND=noninteractive sudo apt install -y \
	capnproto

cargo install just-lsp
