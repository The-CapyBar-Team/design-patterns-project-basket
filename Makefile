build-run:
	docker compose up --build

run:
	docker compose up

stop:
	docker compose down -v

test:
	docker compose -f docker-compose-integration-testing.yml up --build --abort-on-container-exit --exit-code-from basket_service_integration_testing
