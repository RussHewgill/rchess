#!/bin/bash

# https://lczero.org/dev/wiki/testing-guide/

cutechess-cli \
    -tournament gauntlet \
    -concurrency 1 \
    -pgnout out_pgn.pgn \
    -engine conf=rchess tc=0.5+1.0 \
    -engine conf=stockfish tc=0.5+0.4 \
    -each proto=uci \
    -openings file=tables/openings-10ply-100k.pgn \
    -repeat \
    -rounds 10 \
    -games 2


