#ifndef shogi_legality_lite_bindings_h
#define shogi_legality_lite_bindings_h

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


/**
 * How many elements should an array indexed by [`Color`] have?
 *
 * Examples:
 * ```
 * # use shogi_core::Color;
 * // values is long enough so values[color.index()] never panics
 * let mut values = [0; Color::NUM];
 * values[Color::Black.array_index()] = 10;
 * values[Color::White.array_index()] = -10;
 * ```
 * Since: 0.1.2
 */
#define Color_NUM 2

#if defined(DEFINE_EXPERIMENTAL)
#define PieceKind_OPTION_NUM 15
#endif

/**
 * A player.
 *
 * [`Color`] and <code>[Option]<[Color]></code> are both 1-byte data types.
 * Because they are cheap to copy, they implement [`Copy`].
 */
enum Color {
  /**
   * Black, who plays first. Known as `先手` (*sente*).
   *
   * Its representation is 1.
   */
  Black = 1,
  /**
   * White, who plays second. Known as `後手` (*gote*).
   *
   * Its representation is 2.
   */
  White = 2,
};
typedef uint8_t Color;

/**
 * A hand of a single player. A hand is a multiset of unpromoted pieces (except a king).
 *
 * This type can hold up to 255 pieces of each kind, although the rule of shogi prohibits it.
 *
 * Because [`Hand`] is cheap to copy, it implements [`Copy`](https://doc.rust-lang.org/core/marker/trait.Copy.html).
 * Its [`Default`] value is an empty instance.
 */
typedef struct Hand {
  uint8_t _0[8];
} Hand;
/**
 * The number of all valid pieces in hand.
 *
 * Examples:
 * ```
 * # use shogi_core::Hand;
 * assert_eq!(Hand::all_hand_pieces().count(), Hand::NUM_HAND_PIECES);
 * ```
 * Since: 0.1.2
 */
#define Hand_NUM_HAND_PIECES 7

/**
 * C-compatible type for <code>[Option]<[Piece]></code> with defined representations.
 *
 * Valid representations are `0..=14`, and `17..=30`. `0` represents [`None`], `1..=14` represents a black [`Piece`] and `17..=30` represents a white [`Piece`].
 *
 * cbindgen cannot deduce that <code>[Option]<[Piece]></code> can be represented by `uint8_t` in C, so we need to define the bridge type.
 * See: <https://github.com/eqrion/cbindgen/issues/326>
 */
typedef uint8_t OptionPiece;

/**
 * A subset of all squares.
 *
 * Because [`Bitboard`] is cheap to copy, it implements [`Copy`].
 * Its [`Default`] value is an empty instance.
 */
typedef struct Bitboard {
  uint64_t _0[2];
} Bitboard;

/**
 * C-compatible type for <code>[Option]<[CompactMove]></code>.
 *
 * cbindgen cannot deduce that <code>[Option]<[CompactMove]></code> can be represented by `uint16_t` in C, so we need to define the bridge type.
 * See: <https://github.com/eqrion/cbindgen/issues/326>.
 */
typedef uint16_t OptionCompactMove;

/**
 * A position with its move sequence omitted.
 *
 * This data is insufficient for complete legality checking (such as repetition checking),
 * but in most cases it suffices. If you need a complete legality checking, use `Position`.
 *
 * TODO: describe exactly when a position is considered valid
 */
typedef struct PartialPosition {
  Color side;
  uint16_t ply;
  struct Hand hands[2];
  OptionPiece board[81];
  struct Bitboard player_bb[2];
  struct Bitboard piece_bb[NUM];
  OptionCompactMove last_move;
} PartialPosition;

/**
 * A piece + who owns it.
 *
 * [`Piece`] and <code>[Option]<[Piece]></code> are both 1-byte data types.
 * Because they are cheap to copy, they implement [`Copy`].
 *
 * Valid representations are `1..=14`, and `17..=30`. `1..=14` represents a black [`Piece`] and `17..=30` represents a white [`Piece`].
 * Examples:
 * ```
 * use shogi_core::{Color, Piece, PieceKind};
 * assert_eq!(core::mem::size_of::<Piece>(), 1);
 * assert!(Piece::B_P.as_u8() <= 14);
 * ```
 */
typedef uint8_t Piece;
#if defined(DEFINE_EXPERIMENTAL)
/**
 * How many elements should an array indexed by [`Piece`] have?
 *
 * Examples:
 * ```
 * # use shogi_core::{Color, Piece, PieceKind};
 * // values is long enough so values[piece_kind.index()] never panics
 * let mut values = [0; Piece::NUM];
 * values[Piece::W_P.array_index()] = -10;
 * values[Piece::B_L.array_index()] = 25;
 * values[Piece::W_PR.array_index()] = -155;
 * ```
 * This item is experimental: it is subject to change or deletion.
 */
#define Piece_NUM 31
#endif

/**
 * A square.
 *
 * [`Square`] and <code>[Option]<[Square]></code> are both 1-byte data types.
 * Because they are cheap to copy, they implement [`Copy`].
 */
typedef uint8_t Square;
/**
 * How many elements should an array indexed by [`Square`] have?
 *
 * Examples:
 * ```
 * # use shogi_core::{PieceKind, Square};
 * // values is long enough so values[square.index()] never panics
 * let mut values = [None; Square::NUM];
 * values[Square::SQ_5I.array_index()] = Some(PieceKind::King);
 * ```
 * Since: 0.1.2
 */
#define Square_NUM 81

/**
 * Checks if the normal move is legal.
 *
 * `piece` is given as a hint and `position.piece_at(from) == Some(piece)` must hold.
 */
bool legality_normal_check(const struct PartialPosition *position,
                           Piece piece,
                           Square from,
                           Square to);

#endif /* shogi_legality_lite_bindings_h */
