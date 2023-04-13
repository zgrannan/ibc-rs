#!/bin/bash

# Loop 5 times
for i in {1..5}; do
    echo "Run #$i:"
    
    # Measure and print the execution time
    time prusti-rustc --edition=2018 orig.rs
    
    echo "------------------------"
done
