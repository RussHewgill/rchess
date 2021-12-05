#!/bin/bash

# https://lczero.org/dev/wiki/testing-guide/

    # -engine conf=rchess tc=0.5+1.0 \
    # -engine conf=stockfish tc=0.5+0.4 \

    # -engine conf=rchess tc=0.5+0.4 \
    # -engine conf=stockfish tc=0.5+0.4 \

time=0.5
games=10

while getopts t:n: flag
do
    case "$flag" in
        t) time=${OPTARG};;
        n) games=${OPTARG};;
    esac
done

# echo $time
# echo $games

ENGINE2=rchess_prev
# ENGINE2=stockfish

cutechess-cli \
    -tournament gauntlet \
    -concurrency 1 \
    -pgnout out_pgn.pgn \
    -engine conf=rchess st=$time timemargin=50 \
    -engine conf=$ENGINE2 st=$time timemargin=50 \
    -each proto=uci \
    -openings file=tables/openings-10ply-100k.pgn policy=round \
    -repeat \
    -rounds $games \
    -games 2 \
    -draw movenumber=40 movecount=4 score=8 \
    -resign movecount=4 score=500 \
    -sprt elo0=0 elo1=50 alpha=0.05 beta=0.05


