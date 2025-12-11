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
	rm -f backend/mailfang.db
	touch backend/mailfang.db

build_prod:
	docker build -t mailfang:latest .

run_prod:
	docker run -p 3000:3000 -p 2525:2525 mailfang:latest