#!/usr/bin/env sh

append() {
    cat $1 | sed -e '/^#!/d' -e '/^use prusti_contracts/d' -e '/^use crate/d' -e '/^mod /d' >> $2
}

echo "#![allow(dead_code, unused)]" > orig.rs
echo "use prusti_contracts::*;" >> orig.rs
append src/types.rs orig.rs
append src/swap.rs orig.rs
append src/swap_properties.rs orig.rs
append src/main.rs orig.rs

echo "#![allow(dead_code, unused)]" > resource.rs
echo "use prusti_contracts::*;" >> resource.rs
append src/types.rs resource.rs
append src/swap_resource.rs resource.rs
append src/swap_resource_properties.rs resource.rs
append src/main.rs resource.rs
