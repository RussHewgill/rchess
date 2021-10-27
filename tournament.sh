#!/bin/bash

cutechess-cli \
    -tournament gauntlet \
    -concurrency 1 \
    -pgnout out_pgn.pgn \
    -engine conf=rchess tc=0.4+0.4 \
    -engine conf=stockfish tc=0.4+0.4 \
    -draw movenumber=40 \
    -each proto=uci


