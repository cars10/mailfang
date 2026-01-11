compose = docker compose -f compose.yml

build:
	${compose} build

dev: build
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
	cd frontend && npm install && npm run build
	cd backend && cargo build --release --features embed-frontend
	strip backend/target/release/mailfang

build_docker_prod:
	docker build -t cars10/mailfang:latest .

run_docker_prod:
	docker run -p 3000:3000 -p 2525:2525 cars10/mailfang:latest

push_docker_prod:
	docker push cars10/mailfang:latest
