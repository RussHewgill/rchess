
#include <iostream>

using IndexType = std::uint32_t;

enum Square : int {
  SQ_A1, SQ_B1, SQ_C1, SQ_D1, SQ_E1, SQ_F1, SQ_G1, SQ_H1,
  SQ_A2, SQ_B2, SQ_C2, SQ_D2, SQ_E2, SQ_F2, SQ_G2, SQ_H2,
  SQ_A3, SQ_B3, SQ_C3, SQ_D3, SQ_E3, SQ_F3, SQ_G3, SQ_H3,
  SQ_A4, SQ_B4, SQ_C4, SQ_D4, SQ_E4, SQ_F4, SQ_G4, SQ_H4,
  SQ_A5, SQ_B5, SQ_C5, SQ_D5, SQ_E5, SQ_F5, SQ_G5, SQ_H5,
  SQ_A6, SQ_B6, SQ_C6, SQ_D6, SQ_E6, SQ_F6, SQ_G6, SQ_H6,
  SQ_A7, SQ_B7, SQ_C7, SQ_D7, SQ_E7, SQ_F7, SQ_G7, SQ_H7,
  SQ_A8, SQ_B8, SQ_C8, SQ_D8, SQ_E8, SQ_F8, SQ_G8, SQ_H8,
  SQ_NONE,

  SQUARE_ZERO = 0,
  SQUARE_NB   = 64
};

enum File : int {
  FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H, FILE_NB
};

constexpr File file_of(Square s) {
  return File(s & 7);
}

enum Rank : int {
  RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8, RANK_NB
};

enum Color {
  WHITE, BLACK, COLOR_NB = 2
};

enum PieceType {
  NO_PIECE_TYPE, PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING,
  ALL_PIECES = 0,
  PIECE_TYPE_NB = 8
};

enum Piece {
  NO_PIECE,
  W_PAWN = PAWN,     W_KNIGHT, W_BISHOP, W_ROOK, W_QUEEN, W_KING,
  B_PAWN = PAWN + 8, B_KNIGHT, B_BISHOP, B_ROOK, B_QUEEN, B_KING,
  PIECE_NB = 16
};

enum {
  PS_NONE     =  0,
  PS_W_PAWN   =  0,
  PS_B_PAWN   =  1 * SQUARE_NB,
  PS_W_KNIGHT =  2 * SQUARE_NB,
  PS_B_KNIGHT =  3 * SQUARE_NB,
  PS_W_BISHOP =  4 * SQUARE_NB,
  PS_B_BISHOP =  5 * SQUARE_NB,
  PS_W_ROOK   =  6 * SQUARE_NB,
  PS_B_ROOK   =  7 * SQUARE_NB,
  PS_W_QUEEN  =  8 * SQUARE_NB,
  PS_B_QUEEN  =  9 * SQUARE_NB,
  PS_KING     =  10 * SQUARE_NB,
  PS_NB = 11 * SQUARE_NB
};

static constexpr IndexType PieceSquareIndex[COLOR_NB][PIECE_NB] = {
  // convention: W - us, B - them
  // viewed from other side, W and B are reversed
  { PS_NONE, PS_W_PAWN, PS_W_KNIGHT, PS_W_BISHOP, PS_W_ROOK, PS_W_QUEEN, PS_KING, PS_NONE,
    PS_NONE, PS_B_PAWN, PS_B_KNIGHT, PS_B_BISHOP, PS_B_ROOK, PS_B_QUEEN, PS_KING, PS_NONE },
  { PS_NONE, PS_B_PAWN, PS_B_KNIGHT, PS_B_BISHOP, PS_B_ROOK, PS_B_QUEEN, PS_KING, PS_NONE,
    PS_NONE, PS_W_PAWN, PS_W_KNIGHT, PS_W_BISHOP, PS_W_ROOK, PS_W_QUEEN, PS_KING, PS_NONE }
};

inline Square orient(Color perspective, Square s, Square ksq) {
    return Square(int(s) ^ (bool(perspective) * SQ_A8) ^ ((file_of(ksq) < FILE_E) * SQ_H1));
}

static constexpr int KingBuckets[64] = {
  -1, -1, -1, -1, 31, 30, 29, 28,
  -1, -1, -1, -1, 27, 26, 25, 24,
  -1, -1, -1, -1, 23, 22, 21, 20,
  -1, -1, -1, -1, 19, 18, 17, 16,
  -1, -1, -1, -1, 15, 14, 13, 12,
  -1, -1, -1, -1, 11, 10, 9, 8,
  -1, -1, -1, -1, 7, 6, 5, 4,
  -1, -1, -1, -1, 3, 2, 1, 0
};

// Index of a feature for a given king position and another piece on some square
inline IndexType make_index(Color perspective, Square s, Piece pc, Square ksq) {
    Square o_ksq = orient(perspective, ksq, ksq);
    return IndexType(orient(perspective, s, ksq) + PieceSquareIndex[perspective][pc] + PS_NB * KingBuckets[o_ksq]);
}

int main2() {



    //std::cout << "x = " << x << std::endl;

    return 0;

}

int main2() {

    Square x = SQ_E1;

    std::cout << "x = " << x << std::endl;

    Color persp = WHITE;
    //Color persp = BLACK;

    Square ksq = SQ_D1;
    Square sq = SQ_D2;
    Piece pc = B_PAWN;

    IndexType idx = make_index(persp, sq, pc, ksq);

    std::cout << "idx = " << idx << "" << std::endl;

    return 0;
}



