#!/usr/bin/env bash
set -euo pipefail

function go {
   rustc --edition 2018 -Z ast-json-noexpand "$1" | python3 analyze.py
}

find "$1" -type "f" -name "*.rs" | while read -r sourcefile; do
    go "$sourcefile"
done
