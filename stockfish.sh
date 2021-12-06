#!/bin/bash

# printf "position fen $1\ngo perft $2\n"
# printf "position fen $1\ngo perft $2\n" | stockfish

# setoption name EvalFile value /home/me/code/rust/rchess/nn-cdf1785602d6.nnue
# setoption name EvalFile value nn-cdf1785602d6.nnue

printf "position fen $1\neval\n" | stockfish
# printf "position fen $1\neval\n" | stockfish

