#!/bin/bash

# printf "position fen $1\ngo perft $2\n"
# printf "position fen $1\ngo perft $2\n" | stockfish

# BIN=stockfish
# BIN=/home/me/code/builds/chess_engines/Stockfish/src/stockfish
BIN=/home/me/code/builds/Stockfish/src/stockfish

# setoption name EvalFile value /home/me/code/rust/rchess/nn-cdf1785602d6.nnue
# setoption name EvalFile value nn-cdf1785602d6.nnue

# printf "setoption name EvalFile value /home/me/code/rust/rchess/nn-cdf1785602d6.nnue\nposition fen $1\neval\n" | /home/me/code/builds/chess_engines/Stockfish/src/stockfish
# printf "setoption name EvalFile value ../../../../rust/rchess/nn-cdf1785602d6.nnue\nposition fen $1\neval\n" | $BIN
printf "position fen $1\neval\n" | $BIN
# printf "position fen $1\neval\n" | stockfish

