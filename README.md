# CiDiff article data analysis notebook

## Files

* `dataset.csv`: infomations about all the pairs of logs of our dataset
* `annotations.csv`: lines flagged as relevant to understand the failures on the 100 randomly drawn cases
* `benchmark.csv`: metrics about the results of LCS-diff and CiDiff and the whole dataset
* `survey.csv`: user preferences between LCS-diff and CiDif on the 100 randomly drawn cases
* `analysis.ipynb`: notebook containing all the analysis code

## Usage

We use `uv` to manage the python installation. Use `uv sync` in order to install a virtual environment with the good dependencies for the notebook.

We recommend setting-up your notebook IDE to use this virtual environment to execute the notebook.