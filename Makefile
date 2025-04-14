DC = docker compose
FLAGS ?=

run:
	$(DC) up $(FLAGS)

stop:
	$(DC) down $(FLAGS)

test:
	$(eval TEST_NAME_SPECIFIED := $(if $(TEST),TEST_NAME=$(TEST),))
	$(TEST_NAME_SPECIFIED) $(DC) -f docker-compose-integration-testing.yml up \
		--build \
		--abort-on-container-exit \
		--exit-code-from basket_service_integration_testing \
		--remove-orphans \
		--force-recreate \
		$(FLAGS) || true
	$(DC) -f docker-compose-integration-testing.yml down -v
