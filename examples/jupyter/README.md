# Jupyter Example

This directory contains Jupyter notebooks demonstrating Python data science workflows.

## Notebooks

- **notebook.ipynb**: Basic machine learning workflow using pandas, numpy, matplotlib, and scikit-learn
  - Generates synthetic classification data
  - Trains a logistic regression model
  - Evaluates model performance
  - Visualizes results with confusion matrix and feature importance

## Development

This project uses [ruff](https://docs.astral.sh/ruff/) for both formatting and linting.

```bash
# Install dependencies
make install

# Format code
make format

# Lint code
make lint

# Run both format and lint
make all

# Execute notebook cells and save outputs in-place
make execute

# Execute notebook and save to notebook_executed.ipynb (keeps original clean)
make execute-output

# Run the notebook interactively
make run

# Clean up
make clean
```

**Note:** 
- `make execute` modifies the notebook in-place, saving execution outputs directly to `notebook.ipynb`
- `make execute-output` creates a separate `notebook_executed.ipynb` file, keeping the original notebook clean

## Libraries Used

1. Notebook: [jupyter][jupyter]
2. Data Frames: [pandas][pandas] / [polars][polars]
3. Numerical/Scientific Computing: [numpy][numpy] / [scipy][scipy]
4. Visualisation: [matplotlib][matplotlib] / [seaborn][seaborn] / [plotly][plotly]
5. Statistical Modelling: [statsmodels][statsmodels]
6. Machine Learning: [scikit-learn][scikit-learn]
7. Deep Learning: [pytorch][pytorch] / [tensorflow][tensorflow]
8. Gradient Boosting: [xgboost][xgboost] / [lightgbm][lightgbm]

[jupyter]: https://jupyter.org/
[pandas]: https://pandas.pydata.org/
[polars]: https://pola.rs/
[numpy]: https://numpy.org/
[scipy]: https://scipy.org/
[matplotlib]: https://matplotlib.org/
[seaborn]: https://seaborn.pydata.org/
[plotly]: https://plotly.com/python/
[statsmodels]: https://www.statsmodels.org/stable/index.html
[scikit-learn]: https://scikit-learn.org/stable/index.html
[pytorch]: https://pytorch.org/
[tensorflow]: https://www.tensorflow.org/
[xgboost]: https://xgboost.ai/
[lightgbm]: https://lightgbm.readthedocs.io/en/stable/
