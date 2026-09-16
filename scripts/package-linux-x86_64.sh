#!/usr/bin/env bash
# Build the Linux binary consumed by the internal GitLab release pipeline.

set -euo pipefail

output_dir="${1:-dist}"
asset_name="postgres-language-server-linux-x86_64.tar.gz"
target="${PGLS_LINUX_TARGET:-x86_64-unknown-linux-gnu}"
staging_dir="$(mktemp -d)"

cleanup() {
	rm -f "$staging_dir/postgres-language-server"
	rmdir "$staging_dir"
}
trap cleanup EXIT

mkdir -p "$output_dir"

cargo build --locked --release --target "$target" -p pgls_cli

install -m 0755 \
	"target/$target/release/postgres-language-server" \
	"$staging_dir/postgres-language-server"

tar -C "$staging_dir" -czf "$output_dir/$asset_name" postgres-language-server
sha256sum "$output_dir/$asset_name" > "$output_dir/SHA256SUMS"
