# Cidiff Annotation 

An experiment to determine the precision of cidiff to find the relevant lines to debug in a failure log.

## Protocol
The protocol to annotate the logs is explained in `protocol.typ`, and you can generate a pdf with:

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

It will also generate `paths.txt`, the list of 100 paths used to do the annotations.

## GPT/Keyword annotations

To annotate by gpt and keyword run the commands:
```sh
# gpt annotation
cargo run --bin gpt
# keyword annotation
cargo run --bin keyword_search
```

The gpt annotation needs an openai api key in the environment variable `OPENAI_KEY`.

## Visualisation

You can also visualise the annotations produced by the algorithms with:

```sh
cargo run -- <dataset_path> visu <human_path> <merged_path>
```

## Dataset

By default, the program will randomly select a list of 100 pair of logs from the dataset at the given path.
However, you can give it a path to a file containing a list of path to use these instead of the random selection.

```sh
cargo run -- -p <100_paths_file_path> <dataset_path> annotate
```
