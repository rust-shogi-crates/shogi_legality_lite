#![cfg_attr(not(test), no_std)] // Forbids using std::*.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(bench, feature(test))]
#![doc = include_str!("../README.md")]

#[cfg(bench)]
extern crate test;

#[cfg(feature = "alloc")]
extern crate alloc;

use prelegality::will_king_be_captured;
use shogi_core::{
    Bitboard, Color, IllegalMoveKind, LegalityChecker, Move, PartialPosition, Piece, PieceKind,
    PositionStatus, Square,
};

mod normal;
#[doc(hidden)]
#[cfg(feature = "alloc")]
pub mod perft;
mod prelegality;

/// A type for legality checking.
///
/// Methods of this type do not use constant tables.
/// They do not allocate unless it is necessary.
pub struct LiteLegalityChecker;

impl LegalityChecker for LiteLegalityChecker {
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn status(&self, position: &shogi_core::Position) -> PositionStatus {
        let result = self.status_partial(position.inner());
        if result != PositionStatus::InProgress {
            return result;
        }
        // repetition check, takes O(length^2)-time
        let moves = position.moves();
        let length = moves.len();
        let mut history = alloc::vec::Vec::with_capacity(length + 1);
        let mut current = position.initial_position().clone();
        let _ = current.ply_set(1);
        history.push(current.to_sfen_owned());
        let mut repeated = false;
        for i in 0..length {
            let result = current.make_move(moves[i]);
            if result.is_none() {
                return PositionStatus::Invalid;
            }
            let _ = current.ply_set(1);
            history.push(current.to_sfen_owned());
            debug_assert_eq!(history.len(), i + 2);
            let mut eq = 0;
            for j in 0..i + 1 {
                if history[i + 1] == history[j] {
                    eq += 1;
                }
            }
            if eq >= 3 {
                // The same position appeared 4 times.
                repeated = true;
            }
        }
        if repeated {
            return PositionStatus::Draw;
        }
        PositionStatus::InProgress
    }

    fn status_partial(&self, position: &PartialPosition) -> PositionStatus {
        let side = position.side_to_move();
        if crate::prelegality::is_mate(position) == Some(true) {
            return [PositionStatus::WhiteWins, PositionStatus::BlackWins][side.array_index()];
        }
        let hand_b = position.hand_of_a_player(Color::Black);
        let hand_w = position.hand_of_a_player(Color::White);
        let max = [
            (PieceKind::Pawn, 18),
            (PieceKind::Lance, 4),
            (PieceKind::Knight, 4),
            (PieceKind::Silver, 4),
            (PieceKind::Gold, 4),
            (PieceKind::Bishop, 2),
            (PieceKind::Rook, 2),
        ];
        for (piece_kind, limit) in max {
            let mut bb = position.piece_kind_bitboard(piece_kind);
            if let Some(promoted) = piece_kind.promote() {
                bb |= position.piece_kind_bitboard(promoted);
            }
            // Safety: `piece_kind` is valid in hand
            let count = bb.count()
                + unsafe { hand_b.count(piece_kind).unwrap_unchecked() }
                + unsafe { hand_w.count(piece_kind).unwrap_unchecked() };
            if count > limit {
                return PositionStatus::Invalid;
            }
        }
        PositionStatus::InProgress
    }

    fn is_legal_partial(
        &self,
        position: &PartialPosition,
        mv: Move,
    ) -> Result<(), IllegalMoveKind> {
        prelegality::check_with_error(position, mv)?;
        let mut next = position.clone();
        if next.make_move(mv).is_none() {
            return Err(IllegalMoveKind::IncorrectMove);
        }
        if prelegality::will_king_be_captured(&next) == Some(true) {
            return Err(IllegalMoveKind::IgnoredCheck);
        }
        Ok(())
    }

    fn is_legal_partial_lite(&self, position: &PartialPosition, mv: Move) -> bool {
        if !prelegality::check(position, mv) {
            return false;
        }
        let mut next = position.clone();
        if next.make_move(mv).is_none() {
            return false;
        }
        if prelegality::will_king_be_captured(&next) == Some(true) {
            return false;
        }
        true
    }

    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn all_legal_moves_partial(&self, position: &PartialPosition) -> alloc::vec::Vec<Move> {
        use shogi_core::Hand;

        let side = position.side_to_move();
        let my_bb = position.player_bitboard(side);
        let mut result = alloc::vec::Vec::new();
        for from in my_bb {
            let to_candidates = prelegality::normal_from_candidates(position, from);
            for (index, to_candidates) in to_candidates.into_iter().enumerate() {
                let promote = index == 1;
                for to in to_candidates {
                    let mv = Move::Normal { from, to, promote };
                    let mut next = position.clone();
                    if next.make_move(mv).is_none() {
                        continue;
                    }
                    if prelegality::will_king_be_captured(&next) == Some(true) {
                        continue;
                    }
                    result.push(mv);
                }
            }
        }

        let my_hand = position.hand_of_a_player(side);
        if my_hand == Hand::new() {
            return result;
        }
        for piece_kind in shogi_core::Hand::all_hand_pieces() {
            let count = unsafe { my_hand.count(piece_kind).unwrap_unchecked() };
            if count == 0 {
                continue;
            }
            let piece = Piece::new(piece_kind, side);
            for to in position.vacant_bitboard() {
                let mv = Move::Drop { piece, to };
                if self.is_legal_partial_lite(position, mv) {
                    result.push(mv);
                }
            }
        }
        result
    }

    fn normal_from_candidates(&self, position: &PartialPosition, from: Square) -> Bitboard {
        let mut result = Bitboard::empty();
        let side = position.side_to_move();
        let my_bb = position.player_bitboard(side);
        if !my_bb.contains(from) {
            return Bitboard::empty();
        }
        let to_candidates = prelegality::normal_from_candidates(position, from);
        for (index, to_candidates) in to_candidates.into_iter().enumerate() {
            let promote = index == 1;
            for to in to_candidates {
                let mv = Move::Normal { from, to, promote };
                let mut next = position.clone();
                if next.make_move(mv).is_none() {
                    continue;
                }
                if prelegality::will_king_be_captured(&next) == Some(true) {
                    continue;
                }
                result |= to;
            }
        }
        result
    }

    fn normal_to_candidates(
        &self,
        position: &PartialPosition,
        to: Square,
        piece: Piece,
    ) -> Bitboard {
        let mut result = Bitboard::empty();
        for from in Square::all() {
            for promote in [true, false] {
                let mv = Move::Normal { from, to, promote };
                if self.is_legal_partial_lite(position, mv)
                    && position.piece_at(from) == Some(piece)
                {
                    result |= from;
                }
            }
        }
        result
    }

    fn drop_candidates(&self, position: &PartialPosition, piece: Piece) -> Bitboard {
        let mut result = Bitboard::empty();
        for to in Square::all() {
            let mv = Move::Drop { piece, to };
            if self.is_legal_partial_lite(position, mv) {
                result |= to;
            }
        }
        result
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn all_checks_partial(position: &PartialPosition) -> alloc::vec::Vec<Move> {
    use shogi_core::Hand;

    let side = position.side_to_move();
    let king = match position.king_position(side.flip()) {
        Some(x) => x,
        None => return alloc::vec::Vec::new(),
    };
    let king_file = king.file();
    let king_rank = king.rank();
    let my_bb = position.player_bitboard(side);
    let mut result = alloc::vec::Vec::new();
    for from in my_bb {
        let to_candidates = prelegality::normal_from_candidates(position, from);
        for (index, to_candidates) in to_candidates.into_iter().enumerate() {
            let promote = index == 1;
            for to in to_candidates {
                let mv = Move::Normal { from, to, promote };
                let mut next = position.clone();
                if next.make_move(mv).is_none() {
                    continue;
                }
                if prelegality::will_king_be_captured(&next) == Some(true) {
                    continue;
                }
                if is_in_check_partial_lite(&next) {
                    result.push(mv);
                }
            }
        }
    }

    let my_hand = position.hand_of_a_player(side);
    if my_hand == Hand::new() {
        return result;
    }
    for piece_kind in shogi_core::Hand::all_hand_pieces() {
        let count = unsafe { my_hand.count(piece_kind).unwrap_unchecked() };
        if count == 0 {
            continue;
        }
        let bb = all_drop_checks_partial_sub(position, piece_kind, king_file, king_rank);
        for to in bb {
            let mv = Move::Drop {
                piece: Piece::new(piece_kind, side),
                to,
            };
            result.push(mv);
        }
    }
    result
}

#[no_mangle]
pub extern "C" fn all_drop_checks_partial(
    position: &PartialPosition,
    piece_kind: PieceKind,
) -> Bitboard {
    let side = position.side_to_move();
    let my_hand = position.hand_of_a_player(side);
    let count = unsafe { my_hand.count(piece_kind).unwrap_unchecked() };
    if count == 0 {
        return Bitboard::empty();
    }
    let king = match position.king_position(side.flip()) {
        Some(x) => x,
        None => return Bitboard::empty(),
    };
    all_drop_checks_partial_sub(position, piece_kind, king.file(), king.rank())
}

// Does not check if:
// - at least one piece of `piece_kind` is in hand
fn all_drop_checks_partial_sub(
    position: &PartialPosition,
    piece_kind: PieceKind,
    king_file: u8,
    king_rank: u8,
) -> Bitboard {
    let side = position.side_to_move();
    // Special handling for drop pawn mate
    if piece_kind == PieceKind::Pawn {
        let new_rank = match (side, king_rank) {
            (Color::Black, 9) | (Color::White, 1) => {
                return Bitboard::empty();
            }
            (Color::Black, x) => x + 1,
            (Color::White, x) => x - 1,
        };
        // There is at most one candidate square
        let candidate = unsafe { Square::new(king_file, new_rank).unwrap_unchecked() };
        // Is dropping a pawn there legal?
        let mv = Move::Drop {
            piece: Piece::new(PieceKind::Pawn, side),
            to: candidate,
        };
        if prelegality::check(position, mv) {
            return Bitboard::single(candidate);
        } else {
            return Bitboard::empty();
        }
    }
    let piece = Piece::new(piece_kind, side.flip());
    let in_range = crate::normal::from_candidates_without_assertion(
        position.occupied_bitboard(),
        position,
        piece,
        king_file,
        king_rank,
    );
    // If a drop move is stuck, it cannot be a check.
    position.vacant_bitboard() & in_range
}

#[no_mangle]
pub extern "C" fn is_in_check_partial_lite(position: &PartialPosition) -> bool {
    let mut next = position.clone();
    next.side_to_move_set(next.side_to_move().flip());
    matches!(will_king_be_captured(&next), Some(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_legal_moves_partial_works() {
        let position = PartialPosition::startpos();
        let first_moves = LiteLegalityChecker.all_legal_moves_partial(&position);
        assert_eq!(first_moves.len(), 30);
    }

    #[test]
    fn status_works() {
        let position = shogi_core::Position::startpos();
        let result = LiteLegalityChecker.status(&position);
        assert_eq!(result, PositionStatus::InProgress);

        let moves_a = [
            Move::Normal {
                from: Square::SQ_5I,
                to: Square::SQ_5H,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_5A,
                to: Square::SQ_5B,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_5H,
                to: Square::SQ_5I,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_5B,
                to: Square::SQ_5A,
                promote: false,
            },
        ];

        // Slightly different from moves_a
        let moves_b = [
            Move::Normal {
                from: Square::SQ_5I,
                to: Square::SQ_4H,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_5A,
                to: Square::SQ_6B,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_4H,
                to: Square::SQ_5I,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_6B,
                to: Square::SQ_5A,
                promote: false,
            },
        ];
        let mut moves = vec![];
        for _ in 0..3 {
            moves.extend_from_slice(&moves_a);
        }
        let mut position = shogi_core::Position::startpos();
        for mv in moves {
            position.make_move(mv).unwrap();
        }
        let result = LiteLegalityChecker.status(&position);
        assert_eq!(result, PositionStatus::Draw);

        // Even if exactly the same sequence of moves are not made three times, it is considered repetition.
        let mut moves = vec![];
        moves.extend_from_slice(&moves_a);
        moves.extend_from_slice(&moves_b);
        moves.extend_from_slice(&moves_a);
        let mut position = shogi_core::Position::startpos();
        for mv in moves {
            position.make_move(mv).unwrap();
        }
        let result = LiteLegalityChecker.status(&position);
        assert_eq!(result, PositionStatus::Draw);
    }

    #[test]
    fn status_partial_works() {
        let position = PartialPosition::startpos();
        let result = LiteLegalityChecker.status_partial(&position);
        assert_eq!(result, PositionStatus::InProgress);
        // One of the shortest mate sequences
        let moves = [
            Move::Normal {
                from: Square::SQ_2G,
                to: Square::SQ_2F,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_5A,
                to: Square::SQ_4B,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_2F,
                to: Square::SQ_2E,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_4B,
                to: Square::SQ_3B,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_2E,
                to: Square::SQ_2D,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_8B,
                to: Square::SQ_4B,
                promote: false,
            },
            Move::Normal {
                from: Square::SQ_2D,
                to: Square::SQ_2C,
                promote: true,
            },
        ];
        let mut position = PartialPosition::startpos();
        for mv in moves {
            position.make_move(mv).unwrap();
        }
        let result = LiteLegalityChecker.status_partial(&position);
        assert_eq!(result, PositionStatus::BlackWins);
    }

    #[test]
    fn all_checks_partial_works_0() {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate5.psn#L3
        let position =
            PartialPosition::from_usi("sfen 3g1ks2/6g2/4S4/7B1/9/9/9/9/9 b G2rbg2s4n4l18p 1")
                .unwrap();
        let checks = all_checks_partial(&position);
        assert_eq!(checks.len(), 9);
        assert_eq!(checks.iter().filter(|mv| mv.is_drop()).count(), 3);
    }

    #[test]
    fn all_checks_partial_works_1() {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate9.psn#L3
        let position =
            PartialPosition::from_usi("sfen 5kgnl/9/4+B1pp1/8p/9/9/9/9/9 b 2S2rb3g2s3n3l15p 1")
                .unwrap();
        let checks = all_checks_partial(&position);
        assert_eq!(checks.len(), 7);
        assert_eq!(checks.iter().filter(|mv| mv.is_drop()).count(), 3);
    }

    #[test]
    fn all_checks_partial_works_2() {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-19/dpm.psn#L3
        let position =
            PartialPosition::from_usi("sfen 7nk/9/6PB1/6NP1/9/9/9/9/9 b P2rb4g4s2n4l15p 1")
                .unwrap();
        let checks = all_checks_partial(&position);
        assert_eq!(checks.len(), 2);
        assert_eq!(checks.iter().filter(|mv| mv.is_drop()).count(), 0);
    }
}
