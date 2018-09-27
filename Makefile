
# Tried first to cross-compile on my arch linux workstation, but realised that
# there are no suitable toolchain packaged, the one in AUR need to be compiled,
# but is currently even broken. Should get back to this later.

# Then onto using rust-embedded/cross, which uses Docker;
# cargo install cross
# pi3
#TARGET = armv7-unknown-linux-gnueabihf
# pi1 -- works on raspbian, but segfaults on my arch linux install!
#TARGET = arm-unknown-linux-gnueabi
#VARIANT = debug
#build:
#	cross build --target $(TARGET)

# Found another one, which also builds using Docker, that works:

TARGET=arm-unknown-linux-gnueabihf
VARIANT = release
build:
	docker run -it --rm -v $$(pwd):/source \
         -v ~/.cargo/git:/root/.cargo/git \
         -v ~/.cargo/registry:/root/.cargo/registry \
         dlecan/rust-crosscompiler-arm:stable \
         bash -c 'cargo build --release; \
                  chown -R '$(shell id -u):$(shell id -g)' target Cargo.lock; \
                  $$CC_DIR/arm-linux-gnueabihf-strip target/$(TARGET)/$(VARIANT)/klockan'

DUT=toot-wired
deploy:
	rsync -aP target/$(TARGET)/$(VARIANT)/klockan  root@$(DUT):/usr/local/bin/
	rsync -aP klockan.service                      root@$(DUT):/etc/systemd/system/
