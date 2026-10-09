#!/usr/bin/env bash
# Runs once when the dev container is created. Safe to run again.
set -euo pipefail
cd "$(dirname "$0")/.."

# The build output volumes from devcontainer.json start out owned by root.
sudo chown "$(id -u):$(id -g)" target node_modules apps/web/node_modules

sudo corepack enable pnpm

# Picks up a change to rust-toolchain.toml without rebuilding the image.
rustup toolchain install
rustup component add rust-src rust-analyzer

[ -e .env ] || cp .env.example .env
[ -e apps/web/.env ] || cp apps/web/.env.example apps/web/.env

pnpm install --frozen-lockfile
cargo fetch --locked
