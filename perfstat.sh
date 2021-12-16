#!/bin/bash

# EVENTS=cycles,page-faults,instructions,branches,branch-misses,cache-misses,cache-references,alignment-faults
EVENTS=cycles,instructions,\
cpu-clock,task-clock,duration_time,\
page-faults,\
branches,branch-misses,\
cache-references,cache-misses,\
alignment-faults,\
L1-dcache-load-misses,L1-dcache-loads,\
LLC-load-misses,LLC-loads,\
L1-dcache-stores

# L1-icache-load-misses,L1-icache-loads,\

cargo build --bin rchess_engine --release && perf stat -e $EVENTS ./target/release/rchess_engine
# cargo flamegraph -c "record -e $EVENTS -F 997 --call-graph dwarf -g" --bin rchess_engine -- simd

