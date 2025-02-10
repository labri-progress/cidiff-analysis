#!/bin/bash

if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <output_csv_file> <input_csv_file1> <input_csv_file2> ..."
  exit 1
fi

output_file="$1"
shift

head -n 1 "$1" > "$output_file"

for file in "$@"; do
  tail -n +2 "$file" >> "$output_file"
done

echo "Merged CSV files into $output_file successfully."
