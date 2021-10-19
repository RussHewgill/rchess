#!/bin/bash

printf "uci\nucinewgame\nposition fen $1\neval\n" | stockfish

