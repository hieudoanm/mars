SHELL := /bin/sh

.PHONY: all format lint run execute execute-output clean install help

all: format lint

help:
	@echo "Available targets:"
	@echo "  make format          - Format Python code with ruff format"
	@echo "  make lint            - Lint Python code with ruff check"
	@echo "  make execute         - Execute notebook cells and save outputs in-place"
	@echo "  make execute-output  - Execute notebook and save to notebook_executed.ipynb"
	@echo "  make run             - Run Jupyter notebook"
	@echo "  make install         - Install dependencies"
	@echo "  make clean           - Clean generated files"
	@echo "  make all             - Run format and lint"

install:
	@if [ ! -d ".venv" ]; then \
		python3 -m venv .venv; \
	fi
	. .venv/bin/activate && pip install -r requirements.txt

format:
	. .venv/bin/activate && ruff format notebook.ipynb
	. .venv/bin/activate && ruff format .

lint:
	. .venv/bin/activate && ruff check notebook.ipynb
	. .venv/bin/activate && ruff check .

execute:
	. .venv/bin/activate && jupyter nbconvert --to notebook --execute --inplace notebook.ipynb

execute-output:
	. .venv/bin/activate && jupyter nbconvert --to notebook --execute notebook.ipynb --output notebook_executed.ipynb

run:
	. .venv/bin/activate && jupyter notebook notebook.ipynb

clean:
	rm -rf __pycache__
	rm -rf .ipynb_checkpoints
	rm -rf .ruff_cache
	rm -rf .venv
	rm -f notebook_executed.ipynb
