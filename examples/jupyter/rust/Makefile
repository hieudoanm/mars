SHELL := /bin/bash

.PHONY: all run install clean execute execute-output help

all: install

help:
	@echo "Available targets:"
	@echo "  make run            - Run Jupyter notebook with evcxr kernel"
	@echo "  make execute       - Execute notebook cells and save outputs in-place"
	@echo "  make execute-output - Execute notebook and save to notebook_executed.ipynb"
	@echo "  make install       - Install evcxr_jupyter and register kernel"
	@echo "  make clean         - Clean generated files"
	@echo "  make all           - Install dependencies"

install:
	@echo "Installing evcxr_jupyter..."
	cargo install evcxr_jupyter --version 0.21.0
	@echo "Registering evcxr kernel..."
	evcxr_jupyter --install
	@echo "Installing Python dependencies..."
	uv sync
	@echo "Installation complete. Run 'make run' to start the notebook."

run:
	.venv/bin/jupyter notebook notebook.ipynb

execute:
	.venv/bin/jupyter nbconvert --to notebook --execute --inplace notebook.ipynb

execute-output:
	.venv/bin/jupyter nbconvert --to notebook --execute notebook.ipynb --output notebook_executed.ipynb

clean:
	rm -rf __pycache__
	rm -rf .ipynb_checkpoints
	rm -f notebook_executed.ipynb
