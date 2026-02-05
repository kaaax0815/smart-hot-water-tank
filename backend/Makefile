.PHONY: web-dependencies
web-dependencies:
	cd web && pnpm install

.PHONY: web-dev
web-dev:
	cd web && pnpm dev

.PHONY:	container
container:
	docker build -t smart-hot-water-tank .

.PHONY: dev
dev:
	docker compose -f compose.dev.yml up --build
	docker compose -f compose.dev.yml down

.PHONY: sql
sql:
	rm -r database/pkg || true
	docker run --rm -it -v $(PWD):/src -w /src \
	sqlc/sqlc \
	generate

.PHONY: migration
migration:
	docker run --rm -it -v "$(PWD)/database:/database" \
	migrate/migrate create -dir /database/migrations -ext sql $(name)