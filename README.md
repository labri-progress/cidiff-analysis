# Apollo

An experiment to determine the precision of cidiff to find the relevant lines to debug in a failure log.

The protocol is explained in `protocol.typ`, and you can generate a pdf with:

```sh
typst compile protocol.typ
```

## Human annotation

To easily annotate the dataset, run the rust program with:

```sh
cargo run -- <dataset_path>
```

The annotation is automatically saved in `annotation.toml`.

