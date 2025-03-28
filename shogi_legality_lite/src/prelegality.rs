use shogi_core::{
    Bitboard, Color, IllegalMoveKind, Move, PartialPosition, Piece, PieceKind, Square,
};

/// Checks if a move is valid without considering king's safety.
///
/// Since: 0.1.2
pub fn is_valid(position: &PartialPosition, mv: Move) -> bool {
    let side = position.side_to_move();
    match mv {
        Move::Normal { from, to, promote } => {
            // Is `from` occupied by `side`'s piece?
            let from_piece = if let Some(x) = position.piece_at(from) {
                x
            } else {
                return false;
            };
            if from_piece.color() != side {
                return false;
            }
            // Is `to` occupied by `side`'s piece?
            let to_piece = position.piece_at(to);
            if let Some(x) = to_piece {
                if x.color() == side {
                    return false;
                }
                // Capturing king is not allowed.
                if x.piece_kind() == PieceKind::King {
                    return false;
                }
            }
            // Stuck?
            let rel_rank = to.relative_rank(side);
            if rel_rank == 1
                && matches!(
                    from_piece.piece_kind(),
                    PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight,
                )
                && !promote
            {
                return false;
            }
            if rel_rank == 2 && from_piece.piece_kind() == PieceKind::Knight && !promote {
                return false;
            }
            // Can promote?
            if promote && from.relative_rank(side) > 3 && to.relative_rank(side) > 3 {
                return false;
            }
            if promote && from_piece.promote().is_none() {
                return false;
            }
            // Is the move valid?
            crate::normal::check(position, from_piece, from, to)
        }
        Move::Drop { piece, to } => {
            // Does `side` have a piece?
            if piece.color() != side {
                return false;
            }
            let remaining = if let Some(x) = position.hand(piece) {
                x
            } else {
                return false;
            };
            if remaining == 0 {
                return false;
            }
            // Is `to` vacant?
            if position.piece_at(to).is_some() {
                return false;
            }
            // Stuck?
            let rel_rank = to.relative_rank(side);
            if rel_rank == 1
                && matches!(
                    piece.piece_kind(),
                    PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight,
                )
            {
                return false;
            }
            if rel_rank == 2 && piece.piece_kind() == PieceKind::Knight {
                return false;
            }
            // Does a double-pawn (`二歩`, *nifu*) happen?
            if piece.piece_kind() == PieceKind::Pawn {
                let file = to.file();
                for i in 1..=9 {
                    let square = unsafe { Square::new(file, i).unwrap_unchecked() };
                    if position.piece_at(square) == Some(piece) {
                        return false;
                    }
                }
            }
            // Does a drop-pawn-mate (`打ち歩詰め`, *uchifu-zume*) happen?
            if piece.piece_kind() == PieceKind::Pawn {
                let mut next = position.clone();
                let result = next.make_move(mv); // always Some(())
                debug_assert_eq!(result, Some(()));
                if is_mate(&next) == Some(true) {
                    return false;
                }
            }
            true
        }
    }
}

/// Checks if a move is valid without considering king's safety.
/// This function returns a detailed error when `mv` is not legal.
///
/// Since: 0.1.2
pub fn is_valid_with_error(position: &PartialPosition, mv: Move) -> Result<(), IllegalMoveKind> {
    let side = position.side_to_move();
    match mv {
        Move::Normal { from, to, promote } => {
            // Is `from` occupied by `side`'s piece?
            let from_piece = if let Some(x) = position.piece_at(from) {
                x
            } else {
                return Err(IllegalMoveKind::IncorrectMove);
            };
            if from_piece.color() != side {
                return Err(IllegalMoveKind::IncorrectMove);
            }
            // Is `to` occupied by `side`'s piece?
            let to_piece = position.piece_at(to);
            if let Some(x) = to_piece {
                if x.color() == side {
                    return Err(IllegalMoveKind::IncorrectMove);
                }
                // Capturing king is not allowed.
                if x.piece_kind() == PieceKind::King {
                    return Err(IllegalMoveKind::IncorrectMove);
                }
            }
            // Stuck?
            let rel_rank = to.relative_rank(side);
            if rel_rank == 1
                && matches!(
                    from_piece.piece_kind(),
                    PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight,
                )
                && !promote
            {
                return Err(IllegalMoveKind::NormalStuck);
            }
            if rel_rank == 2 && from_piece.piece_kind() == PieceKind::Knight && !promote {
                return Err(IllegalMoveKind::NormalStuck);
            }
            // Can promote?
            if promote && from.relative_rank(side) > 3 && to.relative_rank(side) > 3 {
                return Err(IllegalMoveKind::IncorrectMove);
            }
            if promote && from_piece.promote().is_none() {
                return Err(IllegalMoveKind::IncorrectMove);
            }
            // Is the move valid?
            if crate::normal::check(position, from_piece, from, to) {
                Ok(())
            } else {
                Err(IllegalMoveKind::IncorrectMove)
            }
        }
        Move::Drop { piece, to } => {
            // Does `side` have a piece?
            if piece.color() != side {
                return Err(IllegalMoveKind::IncorrectMove);
            }
            let remaining = if let Some(x) = position.hand(piece) {
                x
            } else {
                return Err(IllegalMoveKind::IncorrectMove);
            };
            if remaining == 0 {
                return Err(IllegalMoveKind::IncorrectMove);
            }
            // Is `to` vacant?
            if position.piece_at(to).is_some() {
                return Err(IllegalMoveKind::IncorrectMove);
            }
            // Stuck?
            let rel_rank = to.relative_rank(side);
            if rel_rank == 1
                && matches!(
                    piece.piece_kind(),
                    PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight,
                )
            {
                return Err(IllegalMoveKind::DropStuck);
            }
            if rel_rank == 2 && piece.piece_kind() == PieceKind::Knight {
                return Err(IllegalMoveKind::DropStuck);
            }
            // Does a double-pawn (`二歩`, *nifu*) happen?
            if piece.piece_kind() == PieceKind::Pawn {
                let file = to.file();
                for i in 1..=9 {
                    let square = unsafe { Square::new(file, i).unwrap_unchecked() };
                    if position.piece_at(square) == Some(piece) {
                        return Err(IllegalMoveKind::TwoPawns);
                    }
                }
            }
            // Does a drop-pawn-mate (`打ち歩詰め`, *uchifu-zume*) happen?
            if piece.piece_kind() == PieceKind::Pawn {
                let mut next = position.clone();
                let result = next.make_move(mv); // always Some(())
                debug_assert_eq!(result, Some(()));
                if is_mate(&next) != Some(false) {
                    return Err(IllegalMoveKind::DropPawnMate);
                }
            }
            Ok(())
        }
    }
}

const FIRST_RANK: Bitboard = {
    let mut result = Bitboard::empty();
    let mut i = 1;
    while i <= 9 {
        result = result.or(unsafe { Bitboard::from_file_unchecked(i, (1 << 8) | 1) });
        i += 1;
    }
    result
};

const FIRST_TWO_RANKS: Bitboard = {
    let mut result = Bitboard::empty();
    let mut i = 1;
    while i <= 9 {
        result =
            result.or(unsafe { Bitboard::from_file_unchecked(i, (1 << 8) | (1 << 7) | 2 | 1) });
        i += 1;
    }
    result
};

const BLACK_PROMOTION: Bitboard = {
    let mut result = Bitboard::empty();
    let mut i = 1;
    while i <= 9 {
        result = result.or(unsafe { Bitboard::from_file_unchecked(i, 7) });
        i += 1;
    }
    result
};

const WHITE_PROMOTION: Bitboard = {
    let mut result = Bitboard::empty();
    let mut i = 1;
    while i <= 9 {
        result = result.or(unsafe { Bitboard::from_file_unchecked(i, 7 << 6) });
        i += 1;
    }
    result
};

pub(crate) fn normal_from_candidates(position: &PartialPosition, from: Square) -> [Bitboard; 2] {
    let side = position.side_to_move();
    let from_piece = if let Some(x) = position.piece_at(from) {
        x
    } else {
        return [Bitboard::empty(); 2];
    };
    if from_piece.color() != side {
        return [Bitboard::empty(); 2];
    }
    // Is the move valid?
    let valid_to = crate::normal::from_candidates(position, from_piece, from);
    // Is `to` occupied by `side`'s piece?
    // Capturing king is not allowed either.
    let my_bb = position.player_bitboard(side);
    let my_bb = my_bb | position.piece_kind_bitboard(PieceKind::King);
    let base = my_bb.andnot(valid_to);
    // Stuck?
    let mut unpromote_prohibited = Bitboard::empty();
    match from_piece.piece_kind() {
        PieceKind::Pawn | PieceKind::Lance => unpromote_prohibited = FIRST_RANK,
        PieceKind::Knight => unpromote_prohibited = FIRST_TWO_RANKS,
        _ => {}
    }
    // Can promote?
    let mut promotable = if from.relative_rank(side) > 3 {
        match side {
            Color::Black => BLACK_PROMOTION,
            Color::White => WHITE_PROMOTION,
        }
    } else {
        !Bitboard::empty()
    };
    if from_piece.promote().is_none() {
        promotable = Bitboard::empty();
    }
    [unpromote_prohibited.andnot(base), base & promotable]
}

/// Returns all valid moves without considering king's safety.
///
/// Since: 0.1.2
pub fn all_valid_moves(position: &PartialPosition) -> impl Iterator<Item = Move> + '_ {
    Square::all()
        .flat_map(|from| {
            Square::all().flat_map(move |to| {
                [false, true]
                    .into_iter()
                    .map(move |promote| Move::Normal { from, to, promote })
            })
        })
        .chain(
            Piece::all()
                .into_iter()
                .flat_map(|piece| Square::all().map(move |to| Move::Drop { piece, to })),
        )
        .filter(|&mv| is_valid(position, mv))
}

/// Can `side` play a move that captures the opponent's king?
///
/// This function returns None if the opponent has no king.
///
/// Since: 0.1.2
pub fn will_king_be_captured(position: &PartialPosition) -> Option<bool> {
    let side = position.side_to_move();
    let occupied = !position.vacant_bitboard();
    let king = position.king_position(side.flip())?;
    let king_file = king.file();
    let king_rank = king.rank();
    let king_peripheral = crate::normal::king(king_file, king_rank);
    let my_bb_peripheral = position.player_bitboard(side) & king_peripheral;
    if !my_bb_peripheral.is_empty() {
        for piece_kind in [PieceKind::King, PieceKind::ProBishop, PieceKind::ProRook] {
            let my_piece = Piece::new(piece_kind, side);
            let piece_bb = position.piece_bitboard(my_piece);
            if !(piece_bb & king_peripheral).is_empty() {
                return Some(true);
            }
        }
        for piece_kind in [
            PieceKind::Pawn,
            PieceKind::Silver,
            PieceKind::Gold,
            PieceKind::ProPawn,
            PieceKind::ProLance,
            PieceKind::ProKnight,
            PieceKind::ProSilver,
        ] {
            let piece = Piece::new(piece_kind, side.flip());
            let my_piece = Piece::new(piece_kind, side);
            let piece_bb = position.piece_bitboard(my_piece);
            let attack = crate::normal::from_candidates_without_assertion(
                occupied, position, piece, king_file, king_rank,
            );
            if !(piece_bb & attack).is_empty() {
                return Some(true);
            }
        }
    }
    // lance, knight
    {
        let my_piece = Piece::new(PieceKind::Lance, side);
        let piece_bb = position.piece_bitboard(my_piece);
        if !piece_bb.is_empty() {
            // from `king`, can `piece` reach a piece of `side` with `piece_kind`?
            let attack = crate::normal::lance_range(side.flip(), king_file, king_rank, occupied);
            if !(attack & piece_bb).is_empty() {
                return Some(true);
            }
        }
        let my_piece = Piece::new(PieceKind::Knight, side);
        let piece_bb = position.piece_bitboard(my_piece);
        if !piece_bb.is_empty() {
            // from `king`, can `piece` reach a piece of `side` with `piece_kind`?
            let attack = crate::normal::knight(side.flip(), king_file, king_rank);
            if !(attack & piece_bb).is_empty() {
                return Some(true);
            }
        }
    }
    macro_rules! ranges {
        ($piece_kind:expr, $pro_piece_kind:expr, $func:expr,) => {
            let my_piece = Piece::new($piece_kind, side);
            let my_pro_piece = Piece::new($pro_piece_kind, side);
            let piece_bb =
                position.piece_bitboard(my_piece) | position.piece_bitboard(my_pro_piece);
            if !piece_bb.is_empty() {
                // from `king`, can `piece` reach a piece of `side` with `piece_kind`?
                let attack = $func(king_file, king_rank, occupied);
                if !(attack & piece_bb).is_empty() {
                    return Some(true);
                }
            }
        };
    }
    ranges!(
        PieceKind::Bishop,
        PieceKind::ProBishop,
        crate::normal::bishop_range,
    );
    ranges!(
        PieceKind::Rook,
        PieceKind::ProRook,
        crate::normal::rook_range,
    );
    Some(false)
}

/// Checks if `side`'s king has no way to escape from being captured.
///
/// This function returns None if `side` has no king.
///
/// For this function to return Some(true), the king does not need to be in check.
///
/// Since: 0.1.2
pub fn is_mate(position: &PartialPosition) -> Option<bool> {
    position.king_position(position.side_to_move())?; // Early return if no king.
    let all = all_valid_moves(position);
    for mv in all {
        let mut next = position.clone();
        let result = next.make_move(mv);
        debug_assert_eq!(result, Some(()));
        if !will_king_be_captured(&next)? {
            return Some(false);
        }
    }
    Some(true)
}

#[cfg(test)]
mod tests {
    use shogi_usi_parser::FromUsi;

    use super::*;

    #[test]
    fn drop_pawn_0() {
        let position =
            PartialPosition::from_usi("sfen 7l1/7pk/7n1/8R/7N1/9/9/9/9 w r2b4g4s2n3l17p 1")
                .unwrap();
        let mv = Move::Drop {
            piece: Piece::new(PieceKind::Pawn, Color::White),
            to: Square::SQ_1C,
        };
        assert!(is_valid(&position, mv));
    }
}
