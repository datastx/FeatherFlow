# Controls the build process of the Feather Flow library.

# Version-related tasks
.PHONY: version bump-version

# Get the current version from Cargo.toml
current-version:
	@grep -m 1 "version" Cargo.toml | awk -F'"' '{print $$2}'

# Bump version in Cargo.toml (usage: make bump-version NEW_VERSION=0.2.0)
bump-version:
	@echo "Current version: $$(grep -m 1 "version" Cargo.toml | awk -F'"' '{print $$2}')"
	@sed -i 's/^version = "[0-9]*\.[0-9]*\.[0-9]*"/version = "$(NEW_VERSION)"/' Cargo.toml
	@echo "Version bumped to $(NEW_VERSION)"
	@echo "Don't forget to commit the changes and create a tag:"
	@echo "git commit -am 'Bump version to $(NEW_VERSION)'"
	@echo "git tag v$(NEW_VERSION)"
	@echo "git push && git push --tags"
