#!/bin/bash

echo "🔧 Baue Hyprclip..."
cargo build --release

BIN_PATH="$HOME/.local/bin"
mkdir -p "$BIN_PATH"
cp target/release/hyprclip "$BIN_PATH"

echo "✅ Hyprclip installiert unter $BIN_PATH/hyprclip"
echo "👉 Füge folgendes zu deiner Waybar-Konfiguration hinzu:"

cat <<EOF
{
  "custom/hyprclip": {
    "format": "{}",
    "exec": "$BIN_PATH/hyprclip --waybar",
    "interval": 2,
    "return-type": "json"
  }
}
EOF
