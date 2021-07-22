#!/usr/bin/env bash
set -euo pipefail

FILENAME="$1"

# if ! grep "extern crate prusti_contracts" "$1" > /dev/null; then
#     sed -i '1s;^;extern crate prusti_contracts\;\n;' "$FILENAME"
# fi

function comment {
    sed -i "s/^\s*$1/\/\/ \0/" "$FILENAME"
}
function delete {
    sed -i "s/$1//" "$FILENAME"
}

function remove_derive {
    sed -i "s/#\[derive(\(.*\), $1\(.*\))/#\[derive(\1\2)/" "$FILENAME"
    sed -i "s/#\[derive($1, \(.*\))/#\[derive(\1)/" "$FILENAME"
}

comment "#\[dyn_clonable::clonable\]"

remove_derive Debug
remove_derive Deserialize
remove_derive Serialize
remove_derive PartialEq
remove_derive Ord
remove_derive PartialOrd
remove_derive Eq

sed -i 's/r#type.try_into/0.try_into/' "$1"

comment "r#type: "
delete ', ::prost::Message'
delete ', ::prost::Oneof'
delete ', ::prost::Enumeration'

comment '#\[derive(::serde.*'
comment '#\[prost.*'
comment '#\[serde.*'
comment '#\[error.*'
comment 'pub r#type.*'
