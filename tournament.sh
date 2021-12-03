#!/bin/bash

# https://lczero.org/dev/wiki/testing-guide/

    # -engine conf=rchess tc=0.5+1.0 \
    # -engine conf=stockfish tc=0.5+0.4 \

    # -engine conf=rchess tc=0.5+0.4 \
    # -engine conf=stockfish tc=0.5+0.4 \

cutechess-cli \
    -tournament gauntlet \
    -concurrency 1 \
    -pgnout out_pgn.pgn \
    -engine conf=rchess st=1.0 timemargin=100 \
    -engine conf=stockfish st=1.0 timemargin=20 \
    -each proto=uci \
    -openings file=tables/openings-10ply-100k.pgn policy=round \
    -repeat \
    -rounds 100 \
    -games 2 \
    -draw movenumber=40 movecount=4 score=8 \
    -resign movecount=4 score=500


