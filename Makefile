all : build

build:
	cargo build

release:
	cargo build --release

install:
	which cargo || (echo "cargo nicht installiert"; exit 1)
	cargo build --release
	sudo install -Dm755 target/release/hyprclip /usr/bin/hyprclip
	sudo install -Dm644 systemd/hyprclip-watcher.service /etc/systemd/system/hyprclip-watcher.service
	sudo systemctl daemon-reload
	sudo systemctl enable hyprclip-watcher.service
	sudo systemctl start hyprclip-watcher.service

check:
	cargo check

clean:
	cargo clean

test:
	cargo test
