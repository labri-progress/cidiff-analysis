# Compute automated annotations (java's ones and keyword)
annotations dataset-path paths-file: 
	just annotation-java {{dataset-path}} {{paths-file}}
	just annotation-keyword {{dataset-path}} {{paths-file}}
	# human annotation is not in the default annotation because it needs human interaction
	# gpt annotation is not in the default annotation recipe because you need an openai api key and it takes quite a long time to run

# Compute java annotations
[working-directory: 'annotator-java']
annotation-java dataset-path paths-file:
	./gradlew shadowJar
	java -jar build/libs/annotator-java-1.0-SNAPSHOT-all.jar {{dataset-path}} {{paths-file}}

# Compute keyword annotation
[working-directory: 'annotator-rust']
annotation-keyword dataset-path paths-file:
	cargo run --bin keyword_search {{dataset-path}} {{paths-file}}

# Compute gpt annotation
[working-directory: 'annotator-rust']
annotation-gpt dataset-path paths-file:
	cargo run --bin gpt {{dataset-path}} {{paths-file}}

# Run human annotation tool
[working-directory: 'annotator-rust']
annotation-human dataset-path:
	cargo run -- {{dataset-path}} annotate -t

# Run the benchmark on all the dataset
[working-directory: 'benchmark']
benchmark dataset-path:
    ./gradlew shadowJar
    java -jar build/libs/benchmark-1.0-SNAPSHOT-all.jar {{dataset-path}}

# Merge csv produced by the annotations
merge-csv:
	./merge_csv.sh csv/annotations.csv annotator-java/selection.csv annotator-rust/gpt.csv annotator-rust/keyword.csv  annotator-rust/annotations.csv
