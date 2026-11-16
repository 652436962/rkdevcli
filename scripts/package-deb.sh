#!/usr/bin/env bash
# Build a Debian package for rkdevcli from already compiled binaries.
#
#   scripts/package-deb.sh [--version V] [--arch ARCH] [--bins DIR] [--out DIR]
#
# Defaults: version from Cargo.toml, arch from uname, binaries from
# target/release, output to dist/. Only dpkg-deb is required; files are owned
# by root via --root-owner-group, so fakeroot is not needed.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="" arch="" bins="$root/target/release" out="$root/dist"

usage() {
  cat <<'EOF'
usage: scripts/package-deb.sh [--version V] [--arch ARCH] [--bins DIR] [--out DIR]

  --version  package version (default: [package] version in Cargo.toml)
  --arch     Debian architecture (default: detected from uname -m)
  --bins     directory containing rkdevcli and rkdev-maintenance
             (default: target/release)
  --out      output directory for the .deb (default: dist/)
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --version) version="${2:?missing value for --version}"; shift 2 ;;
    --arch) arch="${2:?missing value for --arch}"; shift 2 ;;
    --bins) bins="${2:?missing value for --bins}"; shift 2 ;;
    --out) out="${2:?missing value for --out}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [ -z "$version" ]; then
  version="$(awk -F'"' '$1 == "version = " { print $2; exit }' "$root/Cargo.toml")"
fi
if [ -z "$version" ]; then
  echo "could not read version from Cargo.toml; pass --version" >&2
  exit 1
fi

if [ -z "$arch" ]; then
  case "$(uname -m)" in
    x86_64) arch=amd64 ;;
    aarch64|arm64) arch=arm64 ;;
    armv7l|armv6l) arch=armhf ;;
    i386|i686) arch=i386 ;;
    *) echo "unsupported architecture: $(uname -m); pass --arch" >&2; exit 1 ;;
  esac
fi

for bin in rkdevcli rkdev-maintenance; do
  if [ ! -x "$bins/$bin" ]; then
    echo "missing binary: $bins/$bin (build with cargo build --release first)" >&2
    exit 1
  fi
done

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
stage="$work/stage"

install -d "$stage/DEBIAN" "$stage/usr/bin" "$stage/usr/lib/udev/rules.d" \
  "$stage/usr/share/doc/rkdevcli"
install -m 755 "$bins/rkdevcli" "$bins/rkdev-maintenance" "$stage/usr/bin/"
install -m 644 "$root/README.md" "$root/LICENSE" "$stage/usr/share/doc/rkdevcli/"
install -m 644 "$root/LICENSE" "$stage/usr/share/doc/rkdevcli/copyright"
install -m 644 "$root/packaging/99-rkdevcli.rules" "$stage/usr/lib/udev/rules.d/"

size="$(du -sk --exclude=DEBIAN "$stage" | cut -f1)"

cat > "$stage/DEBIAN/control" <<EOF
Package: rkdevcli
Version: $version
Architecture: $arch
Maintainer: fanhui <652436962@qq.com>
Section: utils
Priority: optional
Depends: libc6 (>= 2.35)
Conflicts: rk-dev-suit
Installed-Size: $size
Homepage: https://github.com/652436962/rkdevcli
Description: Rockchip Maskrom/Loader flashing CLI
 rkdevcli speaks the Rockusb protocol directly over USB to scan devices,
 inspect and unpack RKFW/RKAF firmware, flash images or partitions, and
 serve a read-only MCP endpoint for automation.
 .
 It also ships rkdev-maintenance for GPT listing, raw sector reads and
 writes, and Loader container maintenance, plus a udev rule that grants
 non-root access to Rockchip USB devices (VID 2207).
EOF

mkdir -p "$out"
deb="$out/rkdevcli_${version}_${arch}.deb"
dpkg-deb --root-owner-group -Zxz --build "$stage" "$deb"
echo "built $deb"
