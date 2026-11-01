.PHONY: setup install-kernel run execute clean

VENV := .venv
JUPYTER := $(VENV)/bin/jupyter

setup:
	uv venv
	uv pip install jupyterlab

install-kernel:
	cargo install --locked evcxr_jupyter
	evcxr_jupyter --install
	$(JUPYTER) kernelspec list

run:
	$(JUPYTER) lab

execute:
	$(JUPYTER) nbconvert --to notebook --execute rust_demo.ipynb --inplace

clean:
	rm -rf $(VENV) __pycache__ .ipynb_checkpoints
