#!/usr/bin/env bash
rust_command() {
   rustc --edition 2018 -Z ast-json-noexpand "$1" | python3 analyze_derive.py
}

go() {
   while true; do
      OUTPUT=$(rust_command "$1")
      if [ -z "$OUTPUT" ]; then
         return
      fi
      CHAR_NUMBER=$(echo "$OUTPUT" | sed -n 1p)
      ANNOT1=$(echo "$OUTPUT" | sed -n 2p)
      ANNOT2=$(echo "$OUTPUT" | sed -n 3p)
      LINE_NUMBER=$(( $(dd if="$1" bs=1 count="$CHAR_NUMBER" 2>/dev/null | wc -l) + 1 ))
      echo "Line number: $LINE_NUMBER"
      sed -i "$LINE_NUMBER d" "$1"
      sed -i "$LINE_NUMBER i $ANNOT1" "$1"
      sed -i "$LINE_NUMBER i $ANNOT2" "$1"
   done
}

find "$1" -type "f" -name "*.rs" | while read -r sourcefile; do
   echo "Processing $sourcefile"
    go "$sourcefile"
done
