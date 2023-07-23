use raylib::prelude::*;
use ugolki_solver::*;

struct TurnSearchPayload {
    col: PieceColor,
    board: BoardPosition,
    steps: usize,
    depth: usize,
}

const CHECKER_IMG_BYTES: &[u8] = include_bytes!("../assets/checker_white.png");
const MAX_SEARCH_DEPTH: usize = 4;
const MAX_EVAL_DEPTH: usize = 4;
const ANIMATION_STEP: f32 = 0.1;

fn main() {
    raylib::set_trace_log(TraceLogLevel::LOG_WARNING);
    
    let (mut rl, thread) = raylib::init()
        .size(64 * 8, 64 * 8 + 32)
        .title("Ugolki")
        .build();

    let mut board = BoardPosition::default();

    let mut cur_pl = PieceColor::White;
    let player_color: Option<PieceColor> = Some(PieceColor::White);
    let eval_color = player_color.unwrap_or(PieceColor::White);

    let mut searcher = TurnSearcher::new();

    let mut delta = 0.;
    let mut eval_str = String::new();
    let mut turn_str = String::new();
    let mut steps = 0;
    let mut finished = false;
    let mut waiting_for_turn = false;

    let checker_img = Image::load_image_from_mem(
        ".png",
        &CHECKER_IMG_BYTES.to_vec(),
        CHECKER_IMG_BYTES.len() as i32,
    )
    .unwrap();
    let checker_white = rl.load_texture_from_image(&thread, &checker_img).unwrap();

    let mut moved: Option<(PieceColor, Turn, BoardPosition, usize)> = None;
    let mut picked: Option<(PieceColor, Position)> = None;

    let (turn_sender, turn_reciver) = std::sync::mpsc::channel::<Turn>();
    let (payload_sender, payload_reciver) = std::sync::mpsc::channel::<TurnSearchPayload>();

    let _search_thread = std::thread::Builder::new()
        .name("Turn Search".to_owned())
        .spawn(move || loop {
            let Ok(TurnSearchPayload { col, board, steps, depth}) = payload_reciver.recv() else {
                    break;
                };

            let (_ev, turn) = searcher.next_turn(board, col, depth, steps);

            turn_sender.send(turn).unwrap();
        })
        .unwrap();

    let (flip_x, flip_y) = (false, true);

    while !rl.window_should_close() {
        delta += rl.get_frame_time();
        if !board.has_winner(steps) && moved.is_none() {
            if player_color.map(|col| col == cur_pl).unwrap_or_default() {
                if rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
                    // TODO if `displaypos` implements mirroring / rotating board also apply them here
                    let mouse = rl.get_mouse_position() / 64.;

                    let pos_x = (mouse.x.floor() as i32).clamp(0, 7);
                    let pos_y = (mouse.y.floor() as i32).clamp(0, 7);
                    let pos_x = if flip_x { 7-pos_x } else { pos_x }; 
                    let pos_y = if flip_y { 7-pos_y } else { pos_y }; 
            
                    let pos = Position::from_xy(pos_x as u8, pos_y as u8);

                    if let Some((col, picked_pos)) = picked {
                        debug_assert!(col == cur_pl);
                        if board
                            .generate_all_turns_for(cur_pl)
                            .any(|turn| turn.from == picked_pos && turn.to == pos)
                        {
                            let turn = Turn {
                                from: picked_pos,
                                to: pos,
                            };

                            steps += 1;
                            eval_str = format!(
                                "Eval({eval_color}): {}",
                                board.deep_evaluate_for(eval_color, MAX_EVAL_DEPTH, steps)
                            );
                            turn_str = format!("turn {steps}");

                            moved = Some((cur_pl, turn, board, 0));

                            board = board.perform_turn(turn, cur_pl);
                            cur_pl = cur_pl.opposite();
                            delta = 0.;
                        }
                        picked = None;
                    } else if board.0[cur_pl as usize].has_piece_at(pos) {
                        picked = Some((cur_pl, pos));
                    }
                }
            } else {
                let depth = MAX_SEARCH_DEPTH; // if cur_pl == PieceColor::White { 6 } else { 6 };
                if !waiting_for_turn {
                    payload_sender
                        .send(TurnSearchPayload {
                            col: cur_pl,
                            board,
                            steps,
                            depth,
                        })
                        .unwrap();
                    waiting_for_turn = true;
                }

                if let Ok(turn) = turn_reciver.recv_timeout(std::time::Duration::from_millis(12)) {
                    steps += 1;
                    eval_str = format!(
                        "Eval({eval_color}): {}",
                        board.deep_evaluate_for(eval_color, MAX_EVAL_DEPTH, steps)
                    );
                    turn_str = format!("turn {steps}");

                    moved = Some((cur_pl, turn, board, 0));

                    board = board.perform_turn(turn, cur_pl);
                    cur_pl = cur_pl.opposite();
                    delta = 0.;
                    waiting_for_turn = false;
                } else {
                }
            }
        } else if !finished && moved.is_none() {
            eval_str = format!("finished in {steps} turns");
            turn_str.clear();
            finished = true;
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        for ix in 0..8 {
            for iy in 0..8 {
                let col = if (ix ^ iy) & 1 != 0 {
                    Color::GRAY
                } else {
                    Color::DARKGRAY
                };
                d.draw_rectangle(ix * 64, iy * 64, 64, 64, col);
            }
        }

        // TODO option to "mirror" positions for x & y axes
        let displaypos = |pos: Position| {
                let (x, y) = (pos.get_x() as f32 * 64., pos.get_y() as f32 * 64.);
                let x = if flip_x { 7. * 64. - x } else { x }; 
                let y = if flip_y { 7. * 64. - y } else { y }; 
                Vector2::new(x, y)
            };
        let displaypos_i = |pos: Position| {
            let (x, y) = (pos.get_x() as i32 * 64, pos.get_y() as i32 * 64);
            let x = if flip_x { 7 * 64 - x } else { x }; 
            let y = if flip_y { 7 * 64 - y } else { y }; 
            (x, y)
        };

        // draw all static pieces
        for pos in board.0[0].positions_iter() {
            if !matches!(moved, Some((PieceColor::Black, turn, _, _)) if turn.to == pos) {
                d.draw_texture_ex(&checker_white, displaypos(pos), 0., 0.5, Color::DARKGRAY);
            }
        }
        for pos in board.0[1].positions_iter() {
            if !matches!(moved, Some((PieceColor::White, turn, _, _)) if turn.to == pos) {
                d.draw_texture_ex(&checker_white, displaypos(pos), 0., 0.5, Color::WHITE);
            }
        }

        // draw last moved piece
        if let Some((col, turn, board, ref mut i)) = moved {
            let fill_col = if col == PieceColor::White {
                Color::WHITE
            } else {
                Color::DARKGRAY
            };
            let mut visited = false;
            let mut ended = false;

            board.generate_all_turns_seqences_for_pos(col, turn.from, &mut |turns: &[Turn]| {
                if !matches!(turns.last(), Some(turn2) if turn2.to == turn.to) || visited {
                    return;
                }
                visited = true;

                let mut t = (delta / ANIMATION_STEP).min(1.);
                let mut i = *i;

                if i >= turns.len() {
                    ended = true;
                    i = turns.len() - 1;
                    t = 1.;
                }

                let from = displaypos(turns[i].from);
                let to = displaypos(turns[i].to);

                d.draw_texture_ex(
                    &checker_white,
                    from + (to - from) * t + Vector2::new(16. * t * (1. - t), -64. * t * (1. - t)),
                    0.,
                    0.5,
                    fill_col,
                );

                for turn in turns {
                    let from = displaypos(turn.from);
                    let to = displaypos(turn.to);

                    d.draw_line_ex(
                        from + Vector2::new(32., 32.),
                        to + Vector2::new(32., 32.),
                        5.,
                        Color::BLUE,
                    );
                }
            });

            if delta > ANIMATION_STEP {
                delta = 0.;
                *i += 1;
            }

            if ended {
                moved = None;
            }
        }

        // display possible moves
        if let Some((col, pos)) = picked {
            board.generate_all_turns_seqences_for_pos(col, pos, &mut |turns: &[Turn]| {
                let last = turns.last().unwrap();

                let (x, y) = displaypos_i(last.to);
                d.draw_rectangle(x, y, 64, 64, Color::BLUE.fade(0.3));

                for turn in turns {
                    let from = displaypos(turn.from);
                    let to = displaypos(turn.to);
                    d.draw_line_ex(
                        from + Vector2::new(32., 32.),
                        to + Vector2::new(32., 32.),
                        5.,
                        Color::GREEN,
                    );
                }
            });
        }

        d.draw_text(&eval_str, 12, 64 * 8 + 6, 20, Color::RED);
        d.draw_text(&turn_str, 12 + 64 * 6, 64 * 8 + 6, 20, Color::RED);

        if waiting_for_turn {
            d.draw_text("searching...", 12, 6, 60, Color::BLUE);
        }
    }
}
