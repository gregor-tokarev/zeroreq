APP := target/debug/Request Eagle.app

.PHONY: bundle run dev release clean-bundle

# Assemble a debug .app bundle so dev runs get the real bundle identity
# (icon, name, Info.plist) instead of the bare-executable treatment.
bundle:
	cargo build -p request-eagle
	rm -rf "$(APP)"
	mkdir -p "$(APP)/Contents/MacOS" "$(APP)/Contents/Resources"
	cp target/debug/request-eagle "$(APP)/Contents/MacOS/request-eagle"
	cp packaging/macos/AppIcon.icns "$(APP)/Contents/Resources/AppIcon.icns"
	VERSION=$$(sed -n 's/^version = "\(.*\)"/\1/p' crates/request-eagle/Cargo.toml | head -1); \
	BUILD_VERSION=$$(printf %s "$$VERSION" | tr -cd '0-9'); \
	sed -e "s/__VERSION__/$$VERSION/g" -e "s/__BUILD_VERSION__/$${BUILD_VERSION:-1}/g" \
		packaging/macos/Info.plist > "$(APP)/Contents/Info.plist"
	plutil -lint "$(APP)/Contents/Info.plist"
	codesign --force --entitlements packaging/macos/entitlements.plist --sign - "$(APP)"

# Bundle and run in the foreground with logs in the terminal.
run: bundle
	"$(APP)/Contents/MacOS/request-eagle"

# Rebuild the app bundle and restart it whenever project files change.
dev:
	+@./scripts/dev.sh

release:
	./scripts/build-macos.sh

clean-bundle:
	rm -rf "$(APP)"
