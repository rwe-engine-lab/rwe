#!/bin/sh
# Build a local, unsigned Apple Silicon/macOS application bundle.
set -eu
cd "$(dirname "$0")/.."
case "$(uname -s)" in Darwin) ;; *) echo 'macOS is required' >&2; exit 1 ;; esac
bundle_dir=${1:-target/RWE.app}
cargo build --locked --features window,javascript --bin rwe-window
mkdir -p "$bundle_dir/Contents/MacOS" "$bundle_dir/Contents/Resources"
cp target/debug/rwe-window "$bundle_dir/Contents/MacOS/rwe-window"
cp LICENSE WINDOW-DEPENDENCIES.md JS-DEPENDENCIES.md "$bundle_dir/Contents/Resources/"
cat > "$bundle_dir/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>rwe-window</string>
<key>CFBundleIdentifier</key><string>org.rwe-engine.desktop.dev</string>
<key>CFBundleName</key><string>RWE</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSMinimumSystemVersion</key><string>11.0</string>
</dict></plist>
PLIST
printf 'Created %s\n' "$bundle_dir"
