include .env
export

PROJECT := inkarta
DATABASE_FILE = $(PROJECT).db

DEPLOY_HOST := pi@192.168.1.5
DEPLOY_PATH := /home/pi/inkarta/
DEPLOY_TARGET := aarch64-unknown-linux-gnu

.PHONY: *

build:
	cross build --release --target '$(DEPLOY_TARGET)'

clean:
	rm -f $(PROJECT).db*
	cargo clean

database:
	@if [ ! -f "$(DATABASE_FILE)" ]; then \
		echo "Creating $(DATABASE_FILE)..."; \
		sqlx database create --database-url '$(DATABASE_URL)'; \
		sqlite3 '$(DATABASE_FILE)' < schema.sql; \
	fi

deploy:
	ssh '$(DEPLOY_HOST)' "sudo systemctl stop $(PROJECT) || true"
	ssh '$(DEPLOY_HOST)' "mkdir -p $(DEPLOY_PATH)"
	rsync 'target/$(DEPLOY_TARGET)/release/$(PROJECT)' "$(DEPLOY_HOST):$(DEPLOY_PATH)"
	rsync '$(PROJECT).service' "$(DEPLOY_HOST):$(DEPLOY_PATH)"
	ssh "$(DEPLOY_HOST)" "sudo systemctl daemon-reload && sudo systemctl start $(PROJECT)"

develop: database
	bacon run-long

format:
	cargo fmt
	djlint --reformat --format-css --format-js --extension jinja templates/
