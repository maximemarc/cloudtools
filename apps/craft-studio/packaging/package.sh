#!/usr/bin/env bash
# Package a release build of Craft Studio for the OS this runs on, into <out-dir>:
#
#   Linux    craft-studio-<version>-linux-<arch>.tar.gz   binary, .desktop, icons, licenses
#            craft-studio_<version>-1_<debarch>.deb        cargo-deb (dependencies from dpkg-shlibdeps)
#   macOS    craft-studio-<version>-macos-<arch>.dmg       "Craft Studio.app", ad-hoc signed
#   Windows  craft-studio-<version>-windows-x64-setup.exe  Inno Setup installer
#            craft-studio-<version>-windows-x64.zip        portable
#
# Run `cargo build --release --locked -p craft-studio` first. Needs cargo-deb on Linux and
# Inno Setup 6 on Windows (ISCC, or $ISCC). The macOS app is not notarized: Gatekeeper asks
# before the first launch (see release-notes.md).
#
# Usage: apps/craft-studio/packaging/package.sh <out-dir>
set -euo pipefail

[ $# -eq 1 ] || { sed -n '2,15p' "$0" >&2; exit 2; }
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
HERE="$ROOT/apps/craft-studio/packaging"
mkdir -p "$1"
OUT=$(cd "$1" && pwd)
TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target}
# Under target/release so that cargo-deb resolves the assets' "target/release/" paths to it.
WORK="$TARGET_DIR/release/package"

case "$(uname -s)" in
  Linux*) OS=linux EXE=craft-studio ;;
  Darwin*) OS=macos EXE=craft-studio ;;
  MINGW* | MSYS* | CYGWIN*) OS=windows EXE=craft-studio.exe ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac
BIN="$TARGET_DIR/release/$EXE"
[ -x "$BIN" ] || { echo "no $BIN: run cargo build --release --locked -p craft-studio first" >&2; exit 1; }
VERSION=$("$BIN" --version | awk '{print $2}')
SHORT_VERSION=${VERSION%%-*}
ARCH=$(uname -m)
PYTHON=$(command -v python3 || command -v python)
echo "==> Craft Studio $VERSION, $OS $ARCH -> $OUT"

rm -rf "$WORK"
mkdir -p "$WORK"
(cd "$ROOT" && cargo run --release --locked --quiet -p craft-studio --example icons -- "$WORK/icons")
mkdir -p "$WORK/licenses"
cp "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$WORK/licenses/"
"$PYTHON" -I "$ROOT/scripts/third-party-licenses.py" "$WORK/licenses/THIRD-PARTY-LICENSES.txt"

package_linux() {
  local name="craft-studio-$VERSION-linux-$ARCH" stage="$WORK/linux"
  mkdir -p "$stage/$name"
  cp "$BIN" "$HERE/linux/craft-studio.desktop" "$WORK/icons/craft-studio-512.png" "$stage/$name/"
  cp -R "$WORK/licenses" "$stage/$name/"
  tar -C "$stage" -czf "$OUT/$name.tar.gz" "$name"
  # Assets, metadata and dependencies: [package.metadata.deb] in apps/craft-studio/Cargo.toml.
  (cd "$ROOT" && cargo deb -p craft-studio --no-build --no-strip --output "$OUT")
}

package_macos() {
  local app="$WORK/macos/Craft Studio.app" iconset="$WORK/macos/craft-studio.iconset"
  mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$iconset"
  cp "$BIN" "$app/Contents/MacOS/"
  sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/$SHORT_VERSION/g" "$HERE/macos/Info.plist.in" > "$app/Contents/Info.plist"
  plutil -lint "$app/Contents/Info.plist"
  for size in 16 32 128 256 512; do
    cp "$WORK/icons/craft-studio-$size.png" "$iconset/icon_${size}x${size}.png"
    cp "$WORK/icons/craft-studio-$((size * 2)).png" "$iconset/icon_${size}x${size}@2x.png"
  done
  iconutil -c icns -o "$app/Contents/Resources/craft-studio.icns" "$iconset"
  cp -R "$WORK/licenses" "$app/Contents/Resources/"
  # Ad-hoc: Apple silicon only runs signed code, and a bundle signature covers Info.plist and
  # the resources. Notarization needs an Apple Developer ID.
  codesign --force --deep --sign - --timestamp=none "$app"
  codesign --verify --deep --strict "$app"
  local dmg="$WORK/macos/dmg"
  mkdir -p "$dmg"
  mv "$app" "$dmg/"
  ln -s /Applications "$dmg/Applications"
  local file="$OUT/craft-studio-$VERSION-macos-$ARCH.dmg"
  # hdiutil fails now and then on CI runners ("Resource busy").
  for attempt in 1 2 3; do
    hdiutil create -volname "Craft Studio" -srcfolder "$dmg" -ov -format UDZO "$file" && return 0
    echo "hdiutil failed (attempt $attempt)" >&2
    sleep 10
  done
  return 1
}

package_windows() {
  local name="craft-studio-$VERSION-windows-x64" stage="$WORK/windows"
  mkdir -p "$stage"
  cp "$BIN" "$WORK/icons/craft-studio.ico" "$stage/"
  cp -R "$WORK/licenses" "$stage/"
  if command -v 7z > /dev/null; then
    (cd "$stage" && 7z a -tzip -mx=7 "$(cygpath -w "$OUT/$name.zip")" . > /dev/null)
  else
    powershell -NoProfile -Command "Compress-Archive -Path '$(cygpath -w "$stage")\\*' -DestinationPath '$(cygpath -w "$OUT/$name.zip")'"
  fi
  local iscc=${ISCC:-$(command -v iscc || echo "/c/Program Files (x86)/Inno Setup 6/ISCC.exe")}
  "$iscc" /Q "/DAppVersion=$VERSION" "/DStageDir=$(cygpath -w "$stage")" "/DOutputDir=$(cygpath -w "$OUT")" \
    "$(cygpath -w "$HERE/windows/craft-studio.iss")"
}

"package_$OS"
ls -l "$OUT"
