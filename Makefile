all : build

build:
	cargo build

release:
	cargo build --release

install:
	cargo build --release
	sudo cp target/release/hyprclip /usr/bin/
	sudo cp systemd/hyperclip-watcher.service /etc/systemd/system/
	sudo systemctl daemon-reexec
	sudo systemctl daemon-reload
	sudo systemctl enable hyperclip-watcher.service
	sudo systemctl start hyperclip-watcher.service

check:
	cargo check

clean:
	cargo clean

test:
	cargo test
