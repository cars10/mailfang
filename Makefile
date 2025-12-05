compose = docker compose -f compose.yml

build:
	${compose} build

dev:
	${compose} up

ci: lint type-check format test

lint:
	${compose} run --rm frontend npm run lint
	${compose} run --rm backend cargo check

format:
	${compose} run --rm frontend npm run format
	${compose} run --rm backend cargo fmt

type-check:
	${compose} run --rm frontend npm run type-check

test:
	${compose} run --rm backend cargo test

reset_db:
	rm -f backend/mailswallow.db
	touch backend/mailswallow.db