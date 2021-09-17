#!/usr/bin/env bash
rust_command() {
   rustc --edition 2018 -Z ast-json-noexpand "$1" | python3 analyze.py
}

go() {
   while true; do
      CHAR_NUMBER=$(rust_command "$1" | grep 'VERIFIED ' | head -n1 | cut -d ' ' -f 3)
      if [ -z "$CHAR_NUMBER" ]; then
         return
      fi
      LINE_NUMBER=$(( $(dd if="$1" bs=1 count="$CHAR_NUMBER" 2>/dev/null | wc -l) + 1 ))
      sed -i "$LINE_NUMBER i #[cfg_attr(feature=\"prusti_fast\", trusted)]" "$1"
   done
}

find "$1" -type "f" -name "*.rs" | while read -r sourcefile; do
    go "$sourcefile"
done
