#!/bin/sh
# Reproducible local development app + framework fixtures and launchers.
set -eu
cd "$(dirname "$0")/.."
bundle_dir=${1:-target/RWE-apps}
mkdir -p "$bundle_dir"
bundle_dir=$(CDPATH= cd "$bundle_dir" && pwd)
./scripts/bundle-macos.sh "$bundle_dir/RWE.app"
mkdir -p "$bundle_dir/apps"
for app in static-html vue-counter vue-template react-counter; do
    mkdir -p "$bundle_dir/apps/$app"
    cp "examples/$app/index.html" "examples/$app/style.css" "$bundle_dir/apps/$app/"
    if [ -f "examples/$app/app.js" ]; then cp "examples/$app/app.js" "$bundle_dir/apps/$app/"; fi
    if [ -d "examples/$app/vendor" ]; then cp -R "examples/$app/vendor" "$bundle_dir/apps/$app/"; fi
done
cp examples/THIRD-PARTY.md "$bundle_dir/apps/"
cp APPLICATIONS.md "$bundle_dir/"
for pair in Static:static-html Vue:vue-counter Vue-Template:vue-template React:react-counter; do
    label=${pair%%:*}
    page=${pair#*:}
    cat > "$bundle_dir/Open $label.command" <<RUN
#!/bin/sh
set -eu
rwe_demo_dir=\$(CDPATH= cd "\$(dirname "\$0")" && pwd)
exec "\$rwe_demo_dir/RWE.app/Contents/MacOS/rwe-window" "\$rwe_demo_dir/apps/$page/index.html" --watch
RUN
    chmod +x "$bundle_dir/Open $label.command"
done
printf 'Created app examples in %s\n' "$bundle_dir"
