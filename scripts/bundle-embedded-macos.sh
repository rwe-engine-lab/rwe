#!/bin/sh
# Build a relocatable application; HTML/CSS/JS are compiled into its executable.
set -eu
cd "$(dirname "$0")/.."
case "$(uname -s)" in Darwin) ;; *) echo 'macOS is required' >&2; exit 1 ;; esac
example=${2:-embedded-vue}
features=window,javascript
case "$example" in
    embedded-vue) app_name='RWE Embedded Vue' ;;
    keyboard-buttons) app_name='RWE Keyboard Buttons' ;;
    text-input) app_name='RWE Text Input' ;;
    chinese-input) app_name='RWE Chinese Input'; features=window,javascript,fonts ;;
    settings-form) app_name='RWE Settings'; features=window,javascript,fonts ;;
    *) echo 'Supported embedded examples: embedded-vue, keyboard-buttons, text-input, chinese-input, settings-form' >&2; exit 1 ;;
esac
bundle_dir=${1:-target/RWE-Embedded-Vue.app}
cargo build --locked --offline --features "$features" --example "$example"
mkdir -p "$bundle_dir/Contents/MacOS" "$bundle_dir/Contents/Resources"
cp "target/debug/examples/$example" "$bundle_dir/Contents/MacOS/$example"
cp LICENSE CORE-DEPENDENCIES.md REUSE-AUDIT.md WINDOW-DEPENDENCIES.md JS-DEPENDENCIES.md "$bundle_dir/Contents/Resources/"
cp -R licenses/core "$bundle_dir/Contents/Resources/core-licenses"
if [ "$example" = chinese-input ] || [ "$example" = settings-form ]; then cp FONT-DEPENDENCIES.md src/assets/fonts/OFL.txt src/assets/fonts/README.md "$bundle_dir/Contents/Resources/"; fi
if [ "$example" = embedded-vue ] || [ "$example" = settings-form ]; then cp examples/vue-counter/vendor/LICENSE-vue.txt "$bundle_dir/Contents/Resources/"; fi
cat > "$bundle_dir/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>$example</string>
<key>CFBundleIdentifier</key><string>org.rwe-engine.$example.dev</string>
<key>CFBundleName</key><string>$app_name</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSMinimumSystemVersion</key><string>11.0</string>
</dict></plist>
PLIST
printf 'Created %s\n' "$bundle_dir"
