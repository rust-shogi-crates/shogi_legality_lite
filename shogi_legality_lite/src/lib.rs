#![cfg_attr(not(test), no_std)] // Forbids using std::*.
#![cfg_attr(bench, feature(test))]

#[cfg(bench)]
extern crate test;

#[cfg(feature = "alloc")]
extern crate alloc;

use shogi_core::{
    Bitboard, IllegalMoveKind, LegalityChecker, Move, PartialPosition, Piece, PositionStatus,
    Square,
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
        todo!()
    }

    #[allow(unused)]
    fn status_partial(&self, position: &PartialPosition) -> PositionStatus {
        todo!()
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
        for piece_kind in shogi_core::Hand::all_hand_pieces() {
            let piece = Piece::new(piece_kind, side);
            let count = unsafe { position.hand(piece).unwrap_unchecked() };
            if count == 0 {
                continue;
            }
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
}
