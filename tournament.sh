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

# ARGS=$(getopt --longoptions "time,games,out,elo,engine2" -- "$@")
# eval set -- "$ARGS"
# while true; do
    # case "$1" in
        # --time)
            # time="$2"
            # shift 2;;
        # --games)
            # games="$2"
            # shift 2;;
        # --out)
            # OUTPUT_EXTRA="$2"
            # shift 2;;
        # --elo)
            # ELO1="$2"
            # shift 2;;
        # --engine2)
            # ENGINE2="$2"
            # shift 2;;
        # --)
            # break;;
        # *)
            # printf "Unknown option %s\n" "$1"
            # exit 1;;
    # esac
# done

ENGINE2=rchess_prev
# ENGINE2=stockfish
# ENGINE2=arasan
# ENGINE2=gnuchess
# ENGINE2=TSCP
# ENGINE2=rustic

while getopts t:n:e:o: flag
do
    case "$flag" in
        t) time=${OPTARG};;
        n) games=${OPTARG};;
        e) ELO1=${OPTARG};;
        o) OUTPUT_EXTRA=${OPTARG};;
    esac
done

OUTPUT_FILE=out_"$OUTPUT_EXTRA"_pgn_$(date +"%Y-%m-%d_%H:%M:%S").pgn
LOG_FILE=out_"$OUTPUT_EXTRA"_pgn_$(date +"%Y-%m-%d_%H:%M:%S").log

echo output = $OUTPUT_FILE
echo Elo diff = $ELO1, 0.05

# echo $time
# echo $games

    # -engine conf=rchess st=$time timemargin=50 restart=off \
    # -engine conf=$ENGINE2 st=$time timemargin=50 restart=off \

TC="tc=1+0.1"
# TC="st=$time"

echo ENGINE1 = rchess
echo ENGINE2 = $ENGINE2
echo "$TC"

cutechess-cli \
    -tournament gauntlet \
    -concurrency 1 \
    -pgnout $OUTPUT_FILE \
    -engine conf=rchess $TC timemargin=50 restart=off \
    -engine conf=$ENGINE2 $TC timemargin=50 restart=off \
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
    -sprt elo0=$ELO0 elo1=$ELO1 alpha=0.05 beta=0.05 | tee $LOG_FILE


