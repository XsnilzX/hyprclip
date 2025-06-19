# Hyprclip

Hyprclip ist ein minimalistischer, schneller **Clipboard-Manager**📋 für Linux, geschrieben in **Rust**🦀.
Er bietet eine moderne GUI mit [egui](https://github.com/emilk/egui) und nahtlose Integration in **Waybar** über ein JSON-Modul.

## ✨ Features

- 📋 Verlaufsspeicherung des Clipboards
- ⚡ Reaktionsschnelle GUI mit `eframe`/`egui`
- 🧩 JSON-Ausgabe für Integration in Waybar
- 🧼 Minimalistisch, leichtgewichtig & fokussiert auf Performance

## 🛠️ Installation

```bash
git clone https://github.com/XsnilzX/hyprclip.git
cd hyprclip
cargo build --release
```
Die fertige Binary findest du unter target/release/hyprclip.

## 📦 Abhängigkeiten
- Linux mit Wayland (z.B. Hyprland)
- wl-clipboard - für Clipboard zugriff
- Waybar - für Integration in Waybar

## 🚀 Starten
```bash
./target/release/hyprclip
```

Wenn du Hyprclip in Waybar integrieren willst, kannst du das JSON-Modul wie folgt einbinden:
```JSON
"custom/hyprclip": {
  "format": "{}",
  "exec": "~/.cargo/bin/hyprclip --waybar",
  "interval": 1
}
```
(Dieses Beispiel geht davon aus, dass du die Binary global oder via cargo install --path . installiert hast.)

## 📜 Lizenz
Dieses Projekt steht unter der [MIT-Lizenz](LICENSE).
Der Großteil des Codes wurde mit Hilfe von [ChatGPT](https://chatgpt.com/) generiert und anschließend angepasst.

### ✂️ Hyprclip – dein Clipboard, unter Kontrolle.
