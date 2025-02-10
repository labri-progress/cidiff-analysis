# CiDiff article data analysis

## Organisation
This repository is split in 3 projects.
See each project README for more informations.

### Java Project (`annotator-java`)
Generate the annotations csv for the following algorithms:
- cidiff
- lcs-diff
- bigram
- bigram-drain
- cidiff-drainsim

### Rust Project (`annotator-rust`)
Generate the annotations csv for the following algorithms:
- human
- keyword

This project has also a TUI to annotate the logs by a human.

### Python project

The analysis code using the generated CSVs.

* `csv/dataset.csv`: infomations about all the pairs of logs of our dataset
* `csv/annotations.csv`: lines flagged as relevant to understand the failures on the 100 randomly drawn cases
* `csv/benchmark.csv`: metrics about the results of LCS-diff and CiDiff and the whole dataset
* `csv/survey.csv`: user preferences between LCS-diff and CiDif on the 100 randomly drawn cases
* `analysis.ipynb`: notebook containing all the analysis code

#### Usage

We use `uv` to manage the python installation. Use `uv sync` in order to install a virtual environment with the good dependencies for the notebook.

We recommend setting-up your notebook IDE to use this virtual environment to execute the notebook.

