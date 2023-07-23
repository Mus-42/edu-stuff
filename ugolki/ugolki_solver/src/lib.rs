/// Represents a single 8x8 board tile position
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Position(pub u8);

impl Position {
    #[must_use]
    #[inline]
    pub const fn from_xy(x: u8, y: u8) -> Self {
        Self(x | y << 3)
    }

    #[must_use]
    #[inline]
    pub const fn get_x(self) -> u8 {
        self.0 & 7
    }

    #[must_use]
    #[inline]
    pub const fn get_y(self) -> u8 {
        self.0 >> 3
    }

    #[must_use]
    #[inline]
    pub const fn one_up(self) -> Option<Self> {
        if self.0 > 7 {
            Some(Self(self.0 - 8))
        } else {
            None
        }
    }

    #[must_use]
    #[inline]
    pub const fn one_down(self) -> Option<Self> {
        if self.0 < 56 {
            Some(Self(self.0 + 8))
        } else {
            None
        }
    }

    #[must_use]
    #[inline]
    pub const fn one_left(self) -> Option<Self> {
        if self.0 & 7 != 0 {
            Some(Self(self.0 - 1))
        } else {
            None
        }
    }

    #[must_use]
    #[inline]
    pub const fn one_right(self) -> Option<Self> {
        if self.0 & 7 != 7 {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
}

/// Piece color
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PieceColor {
    Black = 0,
    White = 1,
}

impl std::fmt::Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Black => f.write_str("Black"),
            Self::White => f.write_str("White"),
        }
    }
}

impl PieceColor {
    #[must_use]
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

/// Represents set of positions on 8x8 board
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct PositionMask(pub u64);

impl PositionMask {
    pub const WHITE_INITIAL_POSITION: PositionMask = PositionMask(7 | 7 << 8 | 7 << 16);
    pub const BLACK_INITIAL_POSITION: PositionMask =
        PositionMask((224 | 224 << 8 | 224 << 16) << 40);

    #[inline]
    pub fn add_position(&mut self, pos: Position) {
        self.0 |= 1 << pos.0;
    }
    #[inline]
    pub fn remove_position(&mut self, pos: Position) {
        self.0 &= !(1 << pos.0);
    }
    #[must_use]
    #[inline]
    pub const fn has_piece_at(self, pos: Position) -> bool {
        self.0 >> pos.0 & 1 != 0
    }

    #[must_use]
    #[inline]
    pub fn positions_iter(self) -> impl Iterator<Item = Position> {
        (0..64)
            .map(Position)
            .filter(move |pos| self.has_piece_at(*pos))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Turn {
    pub from: Position,
    pub to: Position,
}

impl Turn {
    #[inline]
    pub fn value_for(self, player: PieceColor) -> i32 {
        let dx = self.to.get_x() as i32 - self.from.get_x() as i32;
        let dy = self.to.get_y() as i32 - self.from.get_y() as i32;

        match player {
            PieceColor::Black => -(dx + dy),
            PieceColor::White => dx + dy,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EvaluationResult {
    Defeat { in_steps: i32 },
    Victory { in_steps: i32 },
    InBetween { score: i32 },
}

impl EvaluationResult {
    #[must_use]
    #[inline]
    pub fn opposite(self) -> Self {
        use EvaluationResult::*;
        match self {
            Defeat { in_steps } => Victory { in_steps },
            Victory { in_steps } => Defeat { in_steps },
            InBetween { score } => InBetween { score: -score },
        }
    }

    #[must_use]
    #[inline]
    pub fn add_step(self) -> Self {
        use EvaluationResult::*;
        match self {
            Defeat { in_steps } => Defeat {
                in_steps: in_steps + 1,
            },
            Victory { in_steps } => Victory {
                in_steps: in_steps + 1,
            },
            other => other,
        }
    }

    #[must_use]
    #[inline]
    pub fn is_end(self) -> bool {
        use EvaluationResult::*;
        matches!(self, Defeat { .. } | Victory { .. })
    }
}

impl std::fmt::Display for EvaluationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EvaluationResult::*;
        match self {
            Defeat { in_steps } => f.write_fmt(format_args!("defeat in {in_steps} turns")),
            Victory { in_steps } => f.write_fmt(format_args!("victory in {in_steps} turns")),
            InBetween { score } => f.write_fmt(format_args!("{score}")),
        }
    }
}

use std::cmp::Ordering;

impl PartialOrd for EvaluationResult {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use EvaluationResult::*;
        Some(match (self, other) {
            (InBetween { score: score_a }, InBetween { score: score_b })
            | (Defeat { in_steps: score_a }, Defeat { in_steps: score_b })
            | (Victory { in_steps: score_b }, Victory { in_steps: score_a }) => {
                if *score_a == *score_b {
                    Ordering::Equal
                } else if *score_a < *score_b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }

            (Defeat { .. }, _) => Ordering::Less,
            (Victory { .. }, _) => Ordering::Greater,
            (_, Defeat { .. }) => Ordering::Greater,
            (_, Victory { .. }) => Ordering::Less,
        })
    }
}

impl Ord for EvaluationResult {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// All position for black & white pieces
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoardPosition(pub [PositionMask; 2]);

impl Default for BoardPosition {
    #[inline]
    fn default() -> Self {
        Self([
            PositionMask::BLACK_INITIAL_POSITION,
            PositionMask::WHITE_INITIAL_POSITION,
        ])
    }
}

impl BoardPosition {
    #[must_use]
    #[inline]
    pub fn has_winner(self, turns_count: usize) -> bool {
        self.winner(turns_count).is_some()
    }

    #[must_use]
    #[inline]
    pub fn winner(self, turns_count: usize) -> Option<PieceColor> {
        // works with assumption that get 2 winners - impossible
        if self.0[0] == PositionMask::WHITE_INITIAL_POSITION {
            Some(PieceColor::Black)
        } else if self.0[1] == PositionMask::BLACK_INITIAL_POSITION {
            Some(PieceColor::White)
        } else {
            // prevent "dead" positions
            if turns_count > 50 {
                if (self.0[0].0 & PositionMask::BLACK_INITIAL_POSITION.0) != 0 {
                    Some(PieceColor::White)
                } else if (self.0[1].0 & PositionMask::WHITE_INITIAL_POSITION.0) != 0 {
                    Some(PieceColor::Black)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    #[must_use]
    #[inline]
    pub fn all_pieces_mask(self) -> PositionMask {
        PositionMask(self.0[0].0 | self.0[1].0)
    }

    #[must_use]
    #[inline]
    pub fn is_valid_position(self) -> bool {
        self.0[0].0 & self.0[1].0 == 0
            && self.0[0].0.count_ones() == 9
            && self.0[1].0.count_ones() == 9
    }

    #[must_use]
    #[inline]
    pub fn is_valid_turn(self, turn: Turn, player_color: PieceColor) -> bool {
        self.0[player_color as usize].has_piece_at(turn.from)
            && !self.all_pieces_mask().has_piece_at(turn.to)
            && turn.from != turn.to
        // TODO check that player can move piece between positions
    }

    #[must_use]
    #[inline]
    pub fn perform_turn(self, turn: Turn, player_color: PieceColor) -> Self {
        debug_assert!(self.is_valid_position() && self.is_valid_turn(turn, player_color));
        let mut result = self;
        result.0[player_color as usize].remove_position(turn.from);
        result.0[player_color as usize].add_position(turn.to);
        result
    }

    #[must_use]
    #[inline]
    pub fn generate_all_turns_for(self, player_color: PieceColor) -> impl Iterator<Item = Turn> {
        self.0[player_color as usize]
            .positions_iter()
            .flat_map(move |pos| {
                if player_color == PieceColor::Black {
                    self.generate_turn_for_black(pos)
                } else {
                    self.generate_turn_for_white(pos)
                }
                .positions_iter()
                .map(move |new_pos| Turn {
                    from: pos,
                    to: new_pos,
                })
            })
    }

    // TODO make it Iter<Iter ... >> if possible?
    /// "Display" purpose only (generates long "Jumps" as iterators over turn)
    #[inline]
    pub fn generate_all_turns_seqences_for_pos(
        self,
        player_color: PieceColor,
        pos: Position,
        f: &mut impl FnMut(&[Turn]),
    ) {
        debug_assert!(self.0[player_color as usize].has_piece_at(pos));

        let all_pieces = self.all_pieces_mask();

        // steps

        pos.one_up().map(|new_pos| {
            if !all_pieces.has_piece_at(new_pos) {
                f(&[Turn {
                    from: pos,
                    to: new_pos,
                }]);
            }
        });
        pos.one_down().map(|new_pos| {
            if !all_pieces.has_piece_at(new_pos) {
                f(&[Turn {
                    from: pos,
                    to: new_pos,
                }]);
            }
        });
        pos.one_left().map(|new_pos| {
            if !all_pieces.has_piece_at(new_pos) {
                f(&[Turn {
                    from: pos,
                    to: new_pos,
                }]);
            }
        });
        pos.one_right().map(|new_pos| {
            if !all_pieces.has_piece_at(new_pos) {
                f(&[Turn {
                    from: pos,
                    to: new_pos,
                }]);
            }
        });

        let mut turn_stack = Vec::new();
        let mut visited = PositionMask::default();

        // jumps
        fn search_jumps(
            pos: Position,
            turn_stack: &mut Vec<Turn>,
            visited: &mut PositionMask,
            all_pieces: PositionMask,
            f: &mut impl FnMut(&[Turn]),
        ) {
            if visited.has_piece_at(pos) {
                return;
            }
            visited.add_position(pos);

            pos.one_up().map(|joint_pos| {
                joint_pos.one_up().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        turn_stack.push(Turn {
                            from: pos,
                            to: new_pos,
                        });
                        f(turn_stack);
                        search_jumps(new_pos, turn_stack, visited, all_pieces, f);
                        turn_stack.pop();
                    }
                })
            });
            pos.one_left().map(|joint_pos| {
                joint_pos.one_left().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        turn_stack.push(Turn {
                            from: pos,
                            to: new_pos,
                        });
                        f(turn_stack);
                        search_jumps(new_pos, turn_stack, visited, all_pieces, f);
                        turn_stack.pop();
                    }
                })
            });
            pos.one_down().map(|joint_pos| {
                joint_pos.one_down().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        turn_stack.push(Turn {
                            from: pos,
                            to: new_pos,
                        });
                        f(turn_stack);
                        search_jumps(new_pos, turn_stack, visited, all_pieces, f);
                        turn_stack.pop();
                    }
                })
            });
            pos.one_right().map(|joint_pos| {
                joint_pos.one_right().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        turn_stack.push(Turn {
                            from: pos,
                            to: new_pos,
                        });
                        f(turn_stack);
                        search_jumps(new_pos, turn_stack, visited, all_pieces, f);
                        turn_stack.pop();
                    }
                })
            });
        }

        search_jumps(pos, &mut turn_stack, &mut visited, all_pieces, f);
    }

    #[inline]
    fn generate_turn_for_black(self, from: Position) -> PositionMask {
        let all_pieces = self.all_pieces_mask();
        let mut possible_turns = PositionMask::default();
        let mut visited = PositionMask::default();

        // regular moves

        if let Some(pos) = from.one_up() {
            possible_turns.add_position(pos)
        }
        if let Some(pos) = from.one_left() {
            possible_turns.add_position(pos)
        }
        if let Some(pos) = from.one_down() {
            possible_turns.add_position(pos)
        }
        if let Some(pos) = from.one_right() {
            possible_turns.add_position(pos)
        }

        possible_turns.0 &= !all_pieces.0;

        // jumps
        fn search_jumps(
            pos: Position,
            possible_turns: &mut PositionMask,
            visited: &mut PositionMask,
            all_pieces: PositionMask,
        ) {
            if visited.has_piece_at(pos) {
                return;
            }
            visited.add_position(pos);

            pos.one_up().map(|joint_pos| {
                joint_pos.one_up().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
            pos.one_left().map(|joint_pos| {
                joint_pos.one_left().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
            pos.one_down().map(|joint_pos| {
                joint_pos.one_down().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
            pos.one_right().map(|joint_pos| {
                joint_pos.one_right().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
        }

        search_jumps(from, &mut possible_turns, &mut visited, all_pieces);

        possible_turns
    }
    
    #[inline]
    fn generate_turn_for_white(self, from: Position) -> PositionMask {
        let all_pieces = self.all_pieces_mask();
        let mut possible_turns = PositionMask::default();
        let mut visited = PositionMask::default();

        // regular moves

        if let Some(pos) = from.one_down() {
            possible_turns.add_position(pos)
        }
        if let Some(pos) = from.one_right() {
            possible_turns.add_position(pos)
        }
        if let Some(pos) = from.one_up() {
            possible_turns.add_position(pos)
        }
        if let Some(pos) = from.one_left() {
            possible_turns.add_position(pos)
        }

        possible_turns.0 &= !all_pieces.0;

        // jumps
        fn search_jumps(
            pos: Position,
            possible_turns: &mut PositionMask,
            visited: &mut PositionMask,
            all_pieces: PositionMask,
        ) {
            if visited.has_piece_at(pos) {
                return;
            }
            visited.add_position(pos);

            pos.one_down().map(|joint_pos| {
                joint_pos.one_down().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
            pos.one_right().map(|joint_pos| {
                joint_pos.one_right().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
            pos.one_up().map(|joint_pos| {
                joint_pos.one_up().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
            pos.one_left().map(|joint_pos| {
                joint_pos.one_left().map(|new_pos| {
                    if all_pieces.has_piece_at(joint_pos) && !all_pieces.has_piece_at(new_pos) {
                        possible_turns.add_position(new_pos);
                        search_jumps(new_pos, possible_turns, visited, all_pieces);
                    }
                })
            });
        }

        search_jumps(from, &mut possible_turns, &mut visited, all_pieces);

        possible_turns
    }

    #[must_use]
    #[inline]
    pub fn immediately_evaluate_for(
        self,
        player_color: PieceColor,
        turns_count: usize,
    ) -> EvaluationResult {
        if let Some(winner) = self.winner(turns_count) {
            if winner == player_color {
                return EvaluationResult::Victory { in_steps: 0 };
            } else {
                return EvaluationResult::Defeat { in_steps: 0 };
            }
        }

        let my_val = self.evaluation_helper_position_value(player_color, turns_count);
        let other_val = self.evaluation_helper_position_value(player_color.opposite(), turns_count);

        EvaluationResult::InBetween {
            score: my_val - other_val,
        }
    }

    #[must_use]
    #[inline]
    pub fn deep_evaluate_for(
        self,
        player_color: PieceColor,
        max_depth: usize,
        turns_count: usize,
    ) -> EvaluationResult {
        fn eval_helper(
            board: BoardPosition,
            col: PieceColor,
            cur_col: PieceColor,
            depth: usize,
            turns_count: usize,
        ) -> (Option<usize>, Option<usize>) {
            match board.winner(turns_count) {
                Some(winner) => {
                    return if winner == col {
                        (Some(0), None)
                    } else {
                        (None, Some(0))
                    };
                }
                None => {
                    if depth == 0 {
                        return (None, None);
                    }
                }
            };

            let opposite_col = cur_col.opposite();

            let potisions = board.generate_all_turns_for(cur_col).map(|turn| {
                eval_helper(
                    board.perform_turn(turn, cur_col),
                    col,
                    opposite_col,
                    depth - 1,
                    turns_count + 1,
                )
            });

            let (victory, defeat) = if col == cur_col {
                potisions
                    .map(|(a, b)| (a.map(|v| v + 1), b.map(|v| v + 1)))
                    .fold((None, Some(1)), |(victory, defeat), (new_vic, new_def)| {
                        if let Some(in_steps) = new_vic {
                            (Some(in_steps.min(victory.unwrap_or(in_steps))), None)
                        } else if let Some(in_steps) = new_def {
                            if defeat.is_some() {
                                (victory, Some(in_steps.max(defeat.unwrap_or(in_steps))))
                            } else {
                                (victory, None)
                            }
                        } else {
                            (victory, None)
                        }
                    })
            } else {
                potisions.fold((Some(1), None), |(victory, defeat), (new_vic, new_def)| {
                    if let Some(in_steps) = new_def {
                        (None, Some(in_steps.min(defeat.unwrap_or(in_steps))))
                    } else if let Some(in_steps) = new_vic {
                        if victory.is_some() {
                            (Some(in_steps.max(victory.unwrap_or(in_steps))), defeat)
                        } else {
                            (None, defeat)
                        }
                    } else {
                        (None, defeat)
                    }
                })
            };

            if victory.is_some() && defeat.is_some() {
                (None, None)
            } else {
                (victory, defeat)
            }
        }

        let (victory, defeat) =
            eval_helper(self, player_color, player_color, max_depth, turns_count);

        debug_assert!(victory.is_none() || defeat.is_none());

        if let Some(steps) = victory {
            EvaluationResult::Victory {
                in_steps: steps as i32,
            }
        } else if let Some(steps) = defeat {
            EvaluationResult::Defeat {
                in_steps: steps as i32,
            }
        } else {
            self.immediately_evaluate_for(player_color, turns_count)
        }
    }

    #[inline]
    fn evaluation_helper_position_value(self, color: PieceColor, turns_count: usize) -> i32 {
        let starter_pieces = self.starter_square_pieces(color);
        let dist = self.distance_to_end_point(color);
        let (positional_w_all, positional_w_min) = self.positional_weight(color);

        let positional_w = if turns_count < 30 {
            (positional_w_all + positional_w_min) * 70
        } else {
            positional_w_min * 20
        };

        if self.starter_square_pieces(color) > 0 {
            -dist * 200
                - starter_pieces
                    .pow(2)
                    .saturating_mul((turns_count.max(10) - 10) as i32 * 800)
        } else {
            positional_w - dist * 700 + self.turns_weight(color) * 30
        }
    }

    #[inline]
    fn distance_to_end_point(self, player_color: PieceColor) -> i32 {
        let mut pos_x: [u8; 9] = [0; 9];
        let mut pos_y: [u8; 9] = [0; 9];

        for (i, pos) in self.0[player_color as usize].positions_iter().enumerate() {
            if player_color == PieceColor::Black {
                pos_x[i] = pos.get_x();
                pos_y[i] = pos.get_y();
            } else {
                pos_x[i] = 7 - pos.get_x();
                pos_y[i] = 7 - pos.get_y();
            }
        }

        pos_x.sort_unstable();
        pos_y.sort_unstable();

        let mut score = 0;

        for (i, x) in pos_x.into_iter().enumerate() {
            score += (x.abs_diff(i as u8 / 3) as i32 + 1).pow(3) - 1;
        }
        for (i, y) in pos_y.into_iter().enumerate() {
            score += (y.abs_diff(i as u8 / 3) as i32 + 1).pow(3) - 1;
        }

        score += 15 * pos_x[8] as i32;
        score += 15 * pos_y[8] as i32;

        score
    }

    #[inline]
    fn positional_weight(self, player_color: PieceColor) -> (i32, i32) {
        let mut weight = 0;
        let mut min_weight = 1000000;
        for pos in self.0[player_color as usize].positions_iter() {
            let (x, y) = if player_color == PieceColor::Black {
                (3 - pos.get_x() as i32, 3 - pos.get_y() as i32)
            } else {
                (pos.get_x() as i32 - 4, pos.get_y() as i32 - 4)
            };

            let cur_weight = (x.max(y).pow(3)) * 10 - ((x.abs_diff(y) as i32).max(3) - 3) * 100;

            weight += cur_weight;
            min_weight = min_weight.min(cur_weight);
        }

        (weight, min_weight)
    }

    #[inline]
    fn turns_weight(self, player_color: PieceColor) -> i32 {
        self.0[player_color as usize]
            .positions_iter()
            .map(|from| {
                self.generate_turn_for_black(from)
                    .positions_iter()
                    .map(|to| Turn { from, to }.value_for(player_color) + 2)
                    .max()
                    .unwrap_or(0)
                    .max(0)
            })
            .sum()
    }

    #[inline]
    fn starter_square_pieces(self, player_color: PieceColor) -> i32 {
        let starter_mask = if player_color == PieceColor::Black {
            PositionMask::BLACK_INITIAL_POSITION
        } else {
            PositionMask::WHITE_INITIAL_POSITION
        };

        self.0[player_color as usize]
            .positions_iter()
            .map(|pos| starter_mask.has_piece_at(pos) as i32)
            .sum()
    }
}

impl std::fmt::Display for BoardPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for iy in 0..8 {
            for ix in 0..8 {
                let pos = Position::from_xy(ix, iy);
                if self.0[0].has_piece_at(pos) {
                    f.write_char('B')?;
                } else if self.0[1].has_piece_at(pos) {
                    f.write_char('W')?;
                } else {
                    f.write_char('.')?;
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TurnSearcher {
    cashe: HashMap<(PieceColor, BoardPosition), EvaluationResult>,
    positions_searched: usize,
}

impl TurnSearcher {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            cashe: HashMap::new(),
            positions_searched: 0,
        }
    }

    #[must_use]
    #[inline]
    pub fn next_turn(
        &mut self,
        board: BoardPosition,
        player_color: PieceColor,
        max_depth: usize,
        turns_count: usize,
    ) -> (EvaluationResult, Turn) {
        self.positions_searched = 0;
        self.cashe.clear();
        self.next_turn_initial_impl(board, player_color, max_depth, turns_count)
    }

    #[inline]
    fn next_turn_initial_impl(
        &mut self,
        board: BoardPosition,
        player_color: PieceColor,
        max_depth: usize,
        turns_count: usize,
    ) -> (EvaluationResult, Turn) {
        if max_depth == 1 {
            self.next_turn_impl_at_depht1(board, player_color, turns_count)
        } else {
            let mut alpha = EvaluationResult::Defeat { in_steps: 0 };
            let mut ans_turn = Turn {
                from: Position(0),
                to: Position(0),
            };
            for turn in board.generate_all_turns_for(player_color) {
                let new_board = board.perform_turn(turn, player_color);
                let evaluation = if new_board.has_winner(turns_count + 1) {
                    new_board.immediately_evaluate_for(player_color, turns_count + 1)
                } else {
                    self.next_turn_impl(
                        new_board,
                        player_color.opposite(),
                        max_depth - 1,
                        EvaluationResult::Defeat { in_steps: 0 },
                        alpha.opposite(),
                        turns_count + 1,
                    )
                    .opposite()
                    .add_step()
                };
                if evaluation > alpha {
                    alpha = evaluation;
                    ans_turn = turn;
                }
            }
            (alpha, ans_turn)
        }
    }

    fn next_turn_impl_at_depht1(
        &mut self,
        board: BoardPosition,
        player_color: PieceColor,
        turns_count: usize,
    ) -> (EvaluationResult, Turn) {
        let choose = |(a, av), (b, bv)| if a >= b { (a, av) } else { (b, bv) };

        board
            .generate_all_turns_for(player_color)
            .map(|turn| {
                self.positions_searched += 1;
                (
                    board
                        .perform_turn(turn, player_color)
                        .immediately_evaluate_for(player_color, turns_count + 1),
                    turn,
                )
            })
            .reduce(choose)
            .unwrap_or((
                EvaluationResult::Defeat { in_steps: 0 },
                Turn {
                    from: Position(0),
                    to: Position(0),
                },
            ))
    }

    #[inline]
    fn next_turn_impl(
        &mut self,
        board: BoardPosition,
        player_color: PieceColor,
        max_depth: usize,
        mut alpha: EvaluationResult,
        beta: EvaluationResult,
        turns_count: usize,
    ) -> EvaluationResult {
        if let Some(ev) = self.cashe.get(&(player_color, board)) {
            return *ev;
        }

        let choose = |a: EvaluationResult, b: EvaluationResult| if a >= b { a } else { b };

        let ret = if max_depth == 1 {
            self.next_turn_impl_at_depht1(board, player_color, turns_count)
                .0
        } else {
            let mut turns = board
                .generate_all_turns_for(player_color)
                .collect::<Vec<_>>();
            turns.sort_unstable_by(|a, b| {
                a.value_for(player_color)
                    .cmp(&b.value_for(player_color))
                    .reverse()
            });
            for turn in &turns {
                let new_board = board.perform_turn(*turn, player_color);
                let evaluation = if new_board.has_winner(turns_count) {
                    new_board.immediately_evaluate_for(player_color, turns_count)
                } else {
                    self.next_turn_impl(
                        new_board,
                        player_color.opposite(),
                        max_depth - 1,
                        beta.opposite(),
                        alpha.opposite(),
                        turns_count + 1,
                    )
                    .opposite()
                    .add_step()
                };
                alpha = choose(alpha, evaluation);
                if alpha >= beta {
                    alpha = beta;
                    break;
                }
            }
            alpha
        };

        self.cashe
            .entry((player_color, board))
            .and_modify(|v| *v = choose(*v, ret))
            .or_insert(ret);

        ret
    }
}

impl Default for TurnSearcher {
    fn default() -> Self {
        Self::new()
    }
}
