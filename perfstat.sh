#!/bin/bash

# EVENTS=cycles,page-faults,instructions,branches,branch-misses,cache-misses,cache-references,alignment-faults
EVENTS=cycles,instructions,\
page-faults,\
branches,branch-misses,\
cache-references,cache-misses,\
alignment-faults,\
L1-dcache-load-misses,\
L1-icache-load-misses,\
LLC-load-misses,LLC-loads


cargo build --bin rchess_engine --release && perf stat -e $EVENTS ./target/release/rchess_engine simd

