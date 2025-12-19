.PHONY: demo

# Quick demo: start docker, setup dbt, and run TUI
demo:
	cd examples/postgres && docker compose up -d
	@echo "Waiting for Postgres to be ready..."
	@sleep 5
	cd examples/example_dbt && dbt seed --profiles-dir .
	cd examples/example_dbt && dbt run --profiles-dir .
	cd examples/example_dbt && dbt test --profiles-dir .
	cd examples/example_dbt && dbt compile --profiles-dir .
	@echo "Setup complete! Launching dbt-tui..."
	cargo run
