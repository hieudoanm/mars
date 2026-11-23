TARGET := wasm32-wasip1
OUT := download

.PHONY: build package clean

build:
	cargo build --release --locked --target $(TARGET) -p mars-zed

# Zed builds extensions from source, so the tarball ships the crate plus the
# declarative parts (languages/, grammars/) that extension.toml references.
package: build
	rm -rf $(OUT)
	mkdir -p $(OUT)/pkg
	cp -R extension.toml Cargo.toml src languages grammars $(OUT)/pkg/
	cp ../../Cargo.lock $(OUT)/pkg/Cargo.lock
	tar -czf $(OUT)/extension-zed-mars.tar.gz -C $(OUT)/pkg \
		--exclude='*/node_modules' --exclude='*/target' .
	rm -rf $(OUT)/pkg

clean:
	rm -rf target $(OUT)
