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

# Cidiff Annotation 

An experiment to determine the precision of cidiff to find the relevant lines to debug in a failure log.

## Protocol
The protocol is explained in `protocol.typ`, and you can generate a pdf with:

```sh
typst compile protocol.typ
```

## Human annotation

To easily annotate the dataset, run the rust program with:

```sh
cargo run -- <dataset_path> annotate
```

The annotation is automatically saved in `annotation.toml`.

You can save the annotation as a csv file too with the flag `-t`/`--to-csv` (which you can use later in the visualisation)

## Visualisation

You can also visualise the annotations produced by the algorithms with:

```sh
cargo run -- <dataset_path> visu <human_path> <merged_path>
```

## Dataset

By default, the program will randomly select a list of 100 pair of logs from the dataset at the given path.
However, you can give it a path to a file containing a list of path to use these instead of the random selection.

```sh
# we're using 'ignored' as the dataset path, but it doesn't matter as it will not be used when '-p' is used
cargo run -- -p <100_paths_file_path> ignored annotate
```
