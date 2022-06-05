#![cfg_attr(not(test), no_std)] // Forbids using std::*.
#![cfg_attr(bench, feature(test))]

#[cfg(bench)]
extern crate test;

#[cfg(feature = "alloc")]
extern crate alloc;

use shogi_core::{
    Bitboard, Color, IllegalMoveKind, LegalityChecker, Move, PartialPosition, Piece, PieceKind,
    PositionStatus, Square,
};

mod normal;
#[doc(hidden)]
#[cfg(feature = "alloc")]
pub mod perft;
mod prelegality;

pub struct LiteLegalityChecker;

impl LegalityChecker for LiteLegalityChecker {
    #[allow(unused)]
    #[cfg(feature = "alloc")]
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
        current.ply_set(1);
        history.push(current.to_sfen_owned());
        let mut repeated = false;
        for i in 0..length {
            let result = current.make_move(moves[i]);
            if result.is_none() {
                return PositionStatus::Invalid;
            }
            current.ply_set(1);
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

    #[allow(unused)]
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
        if !prelegality::check(position, mv) {
            return Err(IllegalMoveKind::IncorrectMove);
        }
        let mut next = position.clone();
        if next.make_move(mv).is_none() {}
        if prelegality::will_king_be_captured(&next) != Some(false) {
            return Err(IllegalMoveKind::IncorrectMove);
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
        if prelegality::will_king_be_captured(&next) != Some(false) {
            return false;
        }
        true
    }

    #[cfg(feature = "alloc")]
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
        for to in Square::all() {
            for promote in [true, false] {
                let mv = Move::Normal { from, to, promote };
                if self.is_legal_partial_lite(position, mv) {
                    result |= to;
                }
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
}
