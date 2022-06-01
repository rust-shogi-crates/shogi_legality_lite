use shogi_core::{Bitboard, Color, PartialPosition, Piece, PieceKind, Square};

/// Checks if the normal move is legal.
///
/// `piece` is given as a hint and `position.piece_at(from) == Some(piece)` must hold.
#[allow(unused)]
#[export_name = "legality_normal_check"]
pub extern "C" fn check(
    position: &PartialPosition,
    piece: Piece,
    from: Square,
    to: Square,
) -> bool {
    let attacking = from_candidates(position, piece, from);
    attacking.contains(to)
}

#[inline(always)]
pub fn from_candidates(position: &PartialPosition, piece: Piece, from: Square) -> Bitboard {
    debug_assert_eq!(position.side_to_move(), piece.color());
    debug_assert_eq!(position.piece_at(from), Some(piece));
    let file = from.file();
    let rank = from.rank();
    from_candidates_without_assertion(position, piece, file, rank)
}

pub fn from_candidates_without_assertion(
    position: &PartialPosition,
    piece: Piece,
    file: u8,
    rank: u8,
) -> Bitboard {
    // Is `piece` long-range?
    if matches!(
        piece.piece_kind(),
        PieceKind::Lance
            | PieceKind::Bishop
            | PieceKind::Rook
            | PieceKind::ProBishop
            | PieceKind::ProRook
    ) {
        let occupied = !position.vacant_bitboard();
        let range = match piece.piece_kind() {
            PieceKind::Lance => lance_range(piece.color(), file, rank, occupied),
            PieceKind::Bishop => bishop_range(file, rank, occupied),
            PieceKind::Rook => rook_range(file, rank, occupied),
            PieceKind::ProBishop => bishop_range(file, rank, occupied) | king(file, rank),
            PieceKind::ProRook => rook_range(file, rank, occupied) | king(file, rank),
            _ => unreachable!(),
        };
        return range & !position.player_bitboard(piece.color());
    }
    // `piece` is short-range, i.e., no blocking is possible
    // no need to consider the possibility of blockading by pieces
    let range = unsafe { short_range(piece, file, rank) };
    range & !position.player_bitboard(piece.color())
}

// Safety: `piece` must be short-range, i.e., `piece`'s move cannot be blockaded
unsafe fn short_range(piece: Piece, file: u8, rank: u8) -> Bitboard {
    match piece.piece_kind() {
        PieceKind::Pawn => pawn(piece.color(), Square::new(file, rank).unwrap_unchecked()),
        PieceKind::Knight => knight(piece.color(), file, rank),
        PieceKind::Silver => silver(piece.color(), file, rank),
        PieceKind::Gold
        | PieceKind::ProPawn
        | PieceKind::ProLance
        | PieceKind::ProKnight
        | PieceKind::ProSilver => gold(piece.color(), file, rank),
        PieceKind::King => king(file, rank),
        PieceKind::Lance
        | PieceKind::Bishop
        | PieceKind::Rook
        | PieceKind::ProBishop
        | PieceKind::ProRook => core::hint::unreachable_unchecked(),
    }
}

// If `from` is on the 9th row (i.e., a pawn cannot move), the result is unspecified.
// That being said, since this function is not marked unsafe, this cannot cause UB.
fn pawn(color: Color, from: Square) -> Bitboard {
    let index = from.index();
    match color {
        Color::Black => {
            if index > 1 {
                Bitboard::single(unsafe { Square::from_u8_unchecked(index - 1) })
            } else {
                Bitboard::empty()
            }
        }
        Color::White => {
            if index < 81 {
                Bitboard::single(unsafe { Square::from_u8_unchecked(index + 1) })
            } else {
                Bitboard::empty()
            }
        }
    }
}

fn knight(color: Color, file: u8, rank: u8) -> Bitboard {
    let rrank = match color {
        Color::Black => rank,
        Color::White => 10 - rank,
    };
    if rrank <= 2 {
        return Bitboard::empty();
    }
    let to_rank = match color {
        Color::Black => rank - 2,
        Color::White => rank + 2,
    };
    let mut result = Bitboard::empty();
    if file >= 2 {
        // Safety: file - 1 >= 1, to_rank is in 1..=9
        result |= unsafe { Square::new(file - 1, to_rank).unwrap_unchecked() };
    }
    if file <= 8 {
        // Safety: file + 1 <= 9, to_rank is in 1..=9
        result |= unsafe { Square::new(file + 1, to_rank).unwrap_unchecked() };
    }
    result
}

fn silver(color: Color, file: u8, rank: u8) -> Bitboard {
    let mut result = unsafe {
        Bitboard::from_file_unchecked(
            file,
            match color {
                Color::Black => 1 << rank >> 2,
                Color::White => 1 << rank & 0x1ff,
            },
        )
    };
    let pat = (5 << rank >> 2) & 0x1ff;
    if file >= 2 {
        result |= unsafe { Bitboard::from_file_unchecked(file - 1, pat) };
    }
    if file <= 8 {
        result |= unsafe { Bitboard::from_file_unchecked(file + 1, pat) };
    }
    result
}

fn gold(color: Color, file: u8, rank: u8) -> Bitboard {
    let mut result = unsafe { Bitboard::from_file_unchecked(file, (5 << rank >> 2) & 0x1ff) };
    let pat = match color {
        Color::Black => 3 << rank >> 2,
        Color::White => (3 << (rank - 1)) & 0x1ff,
    };
    if file >= 2 {
        result |= unsafe { Bitboard::from_file_unchecked(file - 1, pat) };
    }
    if file <= 8 {
        result |= unsafe { Bitboard::from_file_unchecked(file + 1, pat) };
    }
    result
}

pub fn king(file: u8, rank: u8) -> Bitboard {
    let mut result = unsafe { Bitboard::from_file_unchecked(file, (5 << rank >> 2) & 0x1ff) };
    if file >= 2 {
        result |= unsafe { Bitboard::from_file_unchecked(file - 1, (7 << rank >> 2) & 0x1ff) };
    }
    if file <= 8 {
        result |= unsafe { Bitboard::from_file_unchecked(file + 1, (7 << rank >> 2) & 0x1ff) };
    }
    result
}

fn lance_range(color: Color, file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    match color {
        Color::Black => long_range_0m1(file, rank, occupied),
        Color::White => long_range_01(file, rank, occupied),
    }
}

fn bishop_range(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let mut result = long_range_1m1(file, rank, occupied) | long_range_11(file, rank, occupied);
    result |= long_range_m1m1(file, rank, occupied);
    result |= long_range_m11(file, rank, occupied);
    result
}

fn rook_range(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let mut result = long_range_0m1(file, rank, occupied)
        | long_range_01(file, rank, occupied)
        | long_range_10(file, rank, occupied);
    result |= long_range_m10(file, rank, occupied);
    result
}

#[allow(unused)]
unsafe fn long_range(
    file: u8,
    rank: u8,
    occupied: Bitboard,
    (file_delta, rank_delta): (i8, i8),
) -> Bitboard {
    let from = Square::new(file, rank).unwrap_unchecked();
    let mut result = Bitboard::empty();
    let mut current = from;
    while let Some(next) = current.shift(file_delta, rank_delta) {
        // `result` includes the blocking piece's square.
        result |= next;
        if occupied.contains(next) {
            break;
        }
        current = next;
    }
    result
}

fn long_range_01(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let occ = occupied.as_u128();
    let step_effect = 0xffffu16.wrapping_shl(rank as u32) & 0x1ff;
    let step_effect = unsafe { Bitboard::from_file_unchecked(file, step_effect) }.as_u128();
    let x = occ & step_effect;
    // Qugiy-style
    let x = (x ^ x.wrapping_sub(1)) & step_effect;
    unsafe { Bitboard::from_u128_unchecked(x) }
}

fn long_range_0m1(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let step_effect = 1u16.wrapping_shl(rank as u32 - 1) - 1;
    let occ_file_pat = unsafe { occupied.get_file_unchecked(file) };
    let x = occ_file_pat & step_effect;
    let x = x | x.wrapping_shr(1);
    let x = x | x.wrapping_shr(2);
    let x = x | x.wrapping_shr(4);
    let x = x.wrapping_shr(1);
    let file_pat = !x & step_effect;
    unsafe { Bitboard::from_file_unchecked(file, file_pat) }
}

/// cbindgen:ignore
const ROW: Bitboard = {
    let mut result = Bitboard::empty();
    let mut file = 1;
    while file <= 9 {
        result = result.or(unsafe { Bitboard::from_file_unchecked(file, 1) });
        file += 1;
    }
    result
};

fn long_range_10(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let occ = occupied.as_u128();
    let step_effect = unsafe { ROW.shift_down(rank - 1).shift_left(file) };
    let step_effect = step_effect.as_u128();
    let x = occ & step_effect;
    // Qugiy-style
    let x = (x ^ x.wrapping_sub(1)) & step_effect;
    unsafe { Bitboard::from_u128_unchecked(x) }
}

fn long_range_m10(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let occ = occupied.as_u128().swap_bytes();
    let step_effect = unsafe { ROW.shift_down(rank - 1).shift_right(10 - file) };
    let step_effect = step_effect.as_u128().swap_bytes();
    let x = occ & step_effect;
    // Qugiy-style
    let x = (x ^ x.wrapping_sub(1)) & step_effect;
    unsafe { Bitboard::from_u128_unchecked(x.swap_bytes()) }
}

/// cbindgen:ignore
const SLASH: Bitboard = {
    let mut result = Bitboard::empty();
    let mut i = 1;
    while i <= 9 {
        result = result.or(unsafe { Bitboard::from_file_unchecked(i, 1 << (i - 1)) });
        i += 1;
    }
    result
};

/// cbindgen:ignore
const BACKSLASH: Bitboard = {
    let mut result = Bitboard::empty();
    let mut i = 1;
    while i <= 9 {
        result = result.or(unsafe { Bitboard::from_file_unchecked(i, 1 << (9 - i)) });
        i += 1;
    }
    result
};

fn long_range_11(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let occ = occupied.as_u128();
    let step_effect = unsafe { SLASH.shift_down(rank).shift_left(file) };
    let step_effect = step_effect.as_u128();
    let x = occ & step_effect;
    // Qugiy-style
    let x = (x ^ x.wrapping_sub(1)) & step_effect;
    unsafe { Bitboard::from_u128_unchecked(x) }
}

fn long_range_1m1(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let occ = occupied.as_u128();
    let step_effect = unsafe { BACKSLASH.shift_up(10 - rank).shift_left(file) };
    let step_effect = step_effect.as_u128();
    let x = occ & step_effect;
    // Qugiy-style
    let x = (x ^ x.wrapping_sub(1)) & step_effect;
    unsafe { Bitboard::from_u128_unchecked(x) }
}

fn long_range_m1m1(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let occ = occupied.as_u128().swap_bytes();
    let step_effect = unsafe { SLASH.shift_up(10 - rank).shift_right(10 - file) };
    let step_effect = step_effect.as_u128().swap_bytes();
    let x = occ & step_effect;
    // Qugiy-style
    let x = (x ^ x.wrapping_sub(1)) & step_effect;
    unsafe { Bitboard::from_u128_unchecked(x.swap_bytes()) }
}

fn long_range_m11(file: u8, rank: u8, occupied: Bitboard) -> Bitboard {
    let occ = occupied.as_u128().swap_bytes();
    let step_effect = unsafe { BACKSLASH.shift_down(rank).shift_right(10 - file) };
    let step_effect = step_effect.as_u128().swap_bytes();
    let x = occ & step_effect;
    // Qugiy-style
    let x = (x ^ x.wrapping_sub(1)) & step_effect;
    unsafe { Bitboard::from_u128_unchecked(x.swap_bytes()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Utility function. If the arguments are out of range, this function panics.
    fn single(file: u8, rank: u8) -> Bitboard {
        Bitboard::single(Square::new(file, rank).unwrap())
    }

    #[test]
    fn pawn_moves_are_correct() {
        let position = PartialPosition::startpos();
        let pawn = Piece::new(PieceKind::Pawn, Color::Black);
        let pawn_square = Square::SQ_7G;
        let attacking = from_candidates(&position, pawn, pawn_square);
        assert_eq!(attacking, single(7, 6));

        // Exhaustive checking: `super::pawn` cannot panic or cause UB
        for color in Color::all() {
            for square in Square::all() {
                if square.relative_rank(color) == 1 {
                    let _ = super::pawn(color, square);
                    continue;
                }
                let result = super::pawn(color, square);
                assert_eq!(result.count(), 1);
            }
        }
        // Compatibility with `flip`
        for square in Square::all() {
            let result_black = super::pawn(Color::Black, square);
            let result_white = super::pawn(Color::White, square.flip());
            assert_eq!(result_white.flip(), result_black);
        }
    }

    #[test]
    fn knight_moves_are_correct() {
        use shogi_core::Move;

        let mut position = PartialPosition::startpos();
        let moves = [
            Move::Normal {
                from: Square::SQ_7G,
                to: Square::SQ_7F,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_3C,
                to: Square::SQ_3D,
                promote: false,
            },
        ];
        for mv in moves {
            position.make_move(mv).unwrap();
        }
        let knight = Piece::new(PieceKind::Knight, Color::Black);
        let knight_square = Square::SQ_8I;
        let attacking = from_candidates(&position, knight, knight_square);
        assert_eq!(attacking, single(7, 7));
    }

    #[test]
    fn silver_moves_are_correct() {
        let position = PartialPosition::startpos();
        let silver = Piece::new(PieceKind::Silver, Color::Black);
        let silver_square = Square::SQ_3I;
        let attacking = from_candidates(&position, silver, silver_square);
        let expected = single(3, 8) | single(4, 8);
        assert_eq!(attacking, expected);

        let expected = single(7, 2) | single(9, 2);
        assert_eq!(super::silver(Color::Black, 8, 1), expected);

        let expected = single(7, 2) | single(8, 2) | single(9, 2);
        assert_eq!(super::silver(Color::White, 8, 1), expected);

        // Exhaustive checking: `super::silver` cannot panic or cause UB
        for color in Color::all() {
            for square in Square::all() {
                let result = super::silver(color, square.file(), square.rank());
                assert!(result.count() <= 5);
            }
        }
        // Compatibility with `flip`
        for square in Square::all() {
            let result_black = super::silver(Color::Black, square.file(), square.rank());
            let result_white =
                super::silver(Color::White, square.flip().file(), square.flip().rank());
            assert_eq!(result_white.flip(), result_black);
        }
    }

    #[test]
    fn gold_moves_are_correct() {
        let position = PartialPosition::startpos();
        let gold = Piece::new(PieceKind::Gold, Color::Black);
        let gold_square = Square::SQ_4I;
        let attacking = from_candidates(&position, gold, gold_square);
        let expected = single(3, 8) | single(4, 8) | single(5, 8);
        assert_eq!(attacking, expected);

        let expected = single(7, 1) | single(8, 2) | single(9, 1);
        assert_eq!(super::gold(Color::Black, 8, 1), expected);

        let expected = single(7, 1) | single(7, 2) | single(8, 2) | single(9, 1) | single(9, 2);
        assert_eq!(super::gold(Color::White, 8, 1), expected);

        // Exhaustive checking: `super::gold` cannot panic or cause UB
        for color in Color::all() {
            for square in Square::all() {
                let result = super::gold(color, square.file(), square.rank());
                assert!(result.count() <= 6);
            }
        }
        // Compatibility with `flip`
        for square in Square::all() {
            let result_black = super::gold(Color::Black, square.file(), square.rank());
            let result_white =
                super::gold(Color::White, square.flip().file(), square.flip().rank());
            assert_eq!(result_white.flip(), result_black);
        }
    }

    #[test]
    fn king_moves_are_correct() {
        let position = PartialPosition::startpos();
        let king = Piece::new(PieceKind::King, Color::Black);
        let king_square = Square::SQ_5I;
        let attacking = from_candidates(&position, king, king_square);
        let expected = single(4, 8) | single(5, 8) | single(6, 8);
        assert_eq!(attacking, expected);
    }

    #[test]
    fn bishop_moves_are_correct() {
        use shogi_core::Move;

        let mut position = PartialPosition::startpos();
        let moves = [
            Move::Normal {
                from: Square::SQ_7G,
                to: Square::SQ_7F,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_3C,
                to: Square::SQ_3D,
                promote: false,
            },
        ];
        for mv in moves {
            position.make_move(mv).unwrap();
        }
        let bishop = Piece::new(PieceKind::Bishop, Color::Black);
        let bishop_square = Square::SQ_8H;
        let attacking = from_candidates(&position, bishop, bishop_square);
        let expected =
            single(2, 2) | single(3, 3) | single(4, 4) | single(5, 5) | single(6, 6) | single(7, 7);
        assert_eq!(attacking, expected);
    }

    #[test]
    fn rook_moves_are_correct() {
        let position = PartialPosition::startpos();
        let rook = Piece::new(PieceKind::Rook, Color::Black);
        let rook_square = Square::SQ_2H;
        let attacking = from_candidates(&position, rook, rook_square);
        let expected =
            single(1, 8) | single(3, 8) | single(4, 8) | single(5, 8) | single(6, 8) | single(7, 8);
        assert_eq!(attacking, expected);
    }
}
