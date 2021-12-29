#!/bin/bash

# https://lczero.org/dev/wiki/testing-guide/

    # -engine conf=rchess tc=0.5+1.0 \
    # -engine conf=stockfish tc=0.5+0.4 \

    # -engine conf=rchess tc=0.5+0.4 \
    # -engine conf=stockfish tc=0.5+0.4 \

time=0.5
games=10
# OUTPUT_FILE=out_pgn.pgn

ELO0=0
ELO1=50

# Hypthesis H0: A is not stronger than B by at least elo1 points
# Hypthesis H1: A is stronger than B by elo0 points

while getopts t:n:e:o: flag
do
    case "$flag" in
        t) time=${OPTARG};;
        n) games=${OPTARG};;
        e) ELO1=${OPTARG};;
        o) OUTPUT_EXTRA=${OPTARG};;
    esac
done

OUTPUT_FILE=out_"$OUTPUT_EXTRA"_pgn_$(date +"%Y-%M-%d_%H:%M:%S").pgn

echo output = $OUTPUT_FILE
echo Elo diff = $ELO1, 0.05

# echo $time
# echo $games

ENGINE2=rchess_prev
# ENGINE2=stockfish

echo ENGINE1=rchess
echo ENGINE2=rchess_prev

cutechess-cli \
    -tournament gauntlet \
    -concurrency 1 \
    -pgnout $OUTPUT_FILE \
    -engine conf=rchess st=$time timemargin=50 restart=on \
    -engine conf=$ENGINE2 st=$time timemargin=50 restart=on \
    -each proto=uci \
    -openings file=tables/openings-10ply-100k.pgn policy=round \
    -tb tables/syzygy/ \
    -tbpieces 5 \
    -repeat \
    -rounds $games \
    -games 2 \
    -draw movenumber=40 movecount=4 score=8 \
    -resign movecount=4 score=500 \
    -ratinginterval 1 \
    -sprt elo0=0 elo1=50 alpha=0.05 beta=0.05


