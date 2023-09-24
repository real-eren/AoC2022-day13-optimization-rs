#!/bin/sh
# this should be run from the root dir of the project 
cargo bench --bench day13_impls -- --format terse --quiet "Day13_A/" > ./assets/full_raw.txt
