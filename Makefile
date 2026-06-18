.DEFAULT_GOAL := build

plugins_dir := /etc/coolercontrol/plugins
service_id := cc-truenas-drivetemp
executable := cc-truenas-drivetemp

.PHONY: clean build test install uninstall

clean:
	cargo clean

build:
	cargo build --release

test:
	cargo test

install: build
	sudo mkdir -p $(plugins_dir)/$(service_id)
	sudo install -m755 ./target/release/$(executable) $(plugins_dir)/$(service_id)/
	sudo install -m644 ./plugin-files/manifest.toml $(plugins_dir)/$(service_id)/
	if [ ! -f "$(plugins_dir)/$(service_id)/config.toml" ]; then sudo install -m640 ./config.example.toml $(plugins_dir)/$(service_id)/config.toml; fi

uninstall:
	sudo rm -rf $(plugins_dir)/$(service_id)
