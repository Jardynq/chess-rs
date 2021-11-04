use core::*;
use std::io::Write;

use crossterm::{ExecutableCommand, QueueableCommand, event, execute, style::{self, Print}};
use arrayvec::ArrayVec;

use bitboard::Bitboard;
use nanorand::Rng;

fn main() {
    for _ in 0..10000 {
        let mut game = GameState::from_fen("r1bqkbnr/pppppppp/2n5/8/PP5/8/2PPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        game.generate_moves(Color::White);
        game.generate_moves(Color::Black);
    }
    return;


    /*
    let mut rng = nanorand::WyRand::new();

    fn to_board(x: u64, y: u64) -> u64 {
        (1u64 << x) << (y * 8u64)
    }
    let mut tor = || -> u64 {
        to_board(rng.generate_range(0..8), rng.generate_range(0..8))
    };
    let mut torc = |count: usize| -> u64 {
        (0..count).fold(0, |board, _| board | tor())
    };

    let mut p = bitboard::PlayerBitboard::default();
    p.bishops = Bitboard(torc(1));

    let oc = Bitboard(0);

    render_bitboard(p.occupancy() | oc, p.attacks(oc));
    return;
    */

    let mut game = GameState::from_fen("r1bqkbnr/pppppppp/2n5/8/PP5/8/2PPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
    let mut game = GameState::from_fen("2bqkbnr/p1pppppp/Pr6/2p4n/3P4/2N5/4PPPP/R1BQKBNR b KQkq - 0 1").unwrap();
    let mut game = GameState::from_fen("4k3/8/4K3/8/8/8/1R6/8 b - - 0 1").unwrap();
    let mut game = GameState::from_fen("8/8/8/4k3/8/2K5/4n3/4R3 w - - 0 1").unwrap();
    // This position has issues with how king thinks a tile is safe
    // If the rook checks the king, then the king thinks that it can simply move right
    // which is illegal
    let mut game = GameState::from_fen("8/5k2/8/1R6/8/8/1K6/8 b - - 0 1").unwrap();
    let mut game = GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let mut game = GameState::from_fen("8/1R3k2/8/8/8/3r4/1K6/8 b - - 0 1").unwrap();
    game.generate_moves(Color::White);
    game.generate_moves(Color::Black);
    //game.validate_moves();

    //render_bitboard(game.black.bitboard.king, game.generate_king_check_mask(Color::Black));
    //return;
    let mut cursor_x = 0;
    let mut cursor_y = 0;

    let mut rendering_disabled = false;
    let mut selected: Option<usize> = None;
    let mut white_is_check = false;
    let mut black_is_check = false;

    let mut invalid_move_count = 0;

    crossterm::terminal::enable_raw_mode().unwrap();
    execute!(
        std::io::stdout(),
        style::Print("\n".repeat(8))
    ).unwrap();
    if !rendering_disabled {
        render(&game, 0, None);
    }
    loop {
        let event = event::read().unwrap();
        //let event = event::Event::Key(event::KeyEvent{code: event::KeyCode::Enter, modifiers: event::KeyModifiers::empty()});
        match event {
            event::Event::Key(event::KeyEvent{code: event::KeyCode::Left, ..}) => {
                if cursor_x > 0 {
                    cursor_x -= 1;
                }
            }
            event::Event::Key(event::KeyEvent{code: event::KeyCode::Right, ..}) => {
                if cursor_x < 8 - 1 {
                    cursor_x += 1;
                }
            }

            event::Event::Key(event::KeyEvent{code: event::KeyCode::Up, ..}) => {
                if cursor_y > 0 {
                    cursor_y -= 1;
                }
            }
            event::Event::Key(event::KeyEvent{code: event::KeyCode::Down, ..}) => {
                if cursor_y < 8 - 1{
                    cursor_y += 1;
                }
            }

            event::Event::Key(event::KeyEvent{code: event::KeyCode::Enter, ..}) => {
                match selected {
                    Some(from) => {
                        selected = None;
                        rendering_disabled = false;

                        let is_valid = match game.current {
                            Color::White => {
                                if let Some(movement) = get_move(&game, Color::White, from, cursor_y * 8 + cursor_x) {
                                    game.play_move_unchecked(movement);
                                    true
                                } else {
                                    false
                                }
                            }
                            Color::Black => {
                                if let Some(movement) = get_move(&game, Color::Black, from, cursor_y * 8 + cursor_x) {
                                    game.play_move_unchecked(movement);
                                    true
                                } else {
                                    false
                                }
                            }
                        };


                        if is_valid {
                            
                            /*if game.is_king_checkmated(Color::White) {
                                println!("White is checkmated! gg :)");
                                break;
                            }
                            if game.is_king_checkmated(Color::Black) {
                                println!("Black is checkmated! gg :)");
                                break;
                            }
                            if game.is_king_stalemated(Color::White) || game.is_king_stalemated(Color::Black) {
                                println!("Stalemate! boring :(");
                                //break;
                            }*/
    
                            game.generate_moves(Color::White);
                            game.generate_moves(Color::Black);
                            //game.validate_moves();
                            game.current = !game.current;
                        }
                    }
                    None => {
                        selected = Some((cursor_y * 8 + cursor_x) as usize);
                    }
                }
            }
            event::Event::Key(event::KeyEvent{code: event::KeyCode::Esc, ..}) => {
                selected = None;
            }

            event::Event::Key(event::KeyEvent{code: event::KeyCode::Char('q'), ..}) => {
                break;
            }

            _ => (),
        }
        

        //let _ = event::read().unwrap();

        if !rendering_disabled {
            render(&game, (cursor_y * 8 + cursor_x) as usize, selected);
        }
        let mat = game.value();
        let white_mat = game.white.value();
        let black_mat = game.white.value();

        //println!("\n{}\nwhite {}\nblack {}", mat, white_mat, black_mat);

        //std::thread::sleep(std::time::Duration::from_millis(250));
    }
    crossterm::terminal::disable_raw_mode().unwrap();
}



pub fn get_move(game: &GameState, color: Color, from: usize, cursor: usize) -> Option<Move> {
    for (_, position, moves) in &game.get_player(color).pieces {
        if position.index() == Some(from) {
            for movement in moves {
                if movement.get_target() == (cursor as u8) {
                    return Some(movement.clone());
                }
            }
        }
    }
    None
}




pub fn render(game: &GameState, cursor: usize, selected: Option<usize>) {
    let mut try_count = 0;
    while let Err(error) = try_render(game, cursor, selected) {
        std::thread::sleep(std::time::Duration::from_millis(500));
        try_count += 1;
        if try_count >= 10 {
            panic!("Failed to render: {}", error);
        }
    }
}

pub fn render_bitboard(position: Bitboard, attacks: Bitboard) {
    let mut buffer = Vec::new();
    execute!(
        buffer,
        crossterm::cursor::MoveTo(0, 0)
    ).unwrap();
    for file in 0..8 {
        for rank in 0..8 {
            let index = file * 8 + rank;
            let color = if (1 << index) & position.0 != 0 {
                style::Color::Rgb{r: 0x21, g: 0x88, b: 0x96}
            } else if (1 << index) & attacks.0 != 0 {
                style::Color::Rgb{r: 0x5b, g: 0x3a, b: 0x21}
            } else {
                style::Color::Rgb{r: 0x85, g: 0x5E, b: 0x42}
            };
            execute!(
                buffer,           
                style::SetBackgroundColor(color),
                style::Print("   "),

                style::ResetColor,
            ).unwrap();
        }
        execute!(
            buffer,
            style::Print("\n")
        ).unwrap();
    }
    if let Ok(result) = String::from_utf8(buffer) {
        execute!(
            std::io::stdout(),
            style::Print(result),
        ).unwrap();
    }
}

pub fn try_render(game: &GameState, cursor: usize, selected: Option<usize>) -> Result<(), std::io::Error> {
    let chose_cursor = cursor;
    let cursor = if let Some(selected) = selected { selected } else { cursor };
    
    let mut selected_moves = ArrayVec::new();
    let mut selected_move_length = 0;

    for (_, board_index, moves) in game.white.pieces.iter() {
        if Some(cursor) == board_index.index() {
            selected_moves = moves.clone();
            selected_move_length = moves.len();
            break;
        }
    }
    for (_, board_index, moves) in game.black.pieces.iter() {
        if Some(cursor) == board_index.index() {
            selected_moves = moves.clone();
            selected_move_length = moves.len();
            break;
        }
    }



    let mut buffer = Vec::new();
    execute!(
        buffer,
        crossterm::cursor::MoveTo(0, 0)
    )?;
    for rank in 0..8 {
        for file in 0..8 {
            let index = rank * 8 + file;
            let tile = game.board[index];
            let is_odd = (file % 2 == 0) ^ (rank % 2 == 0);

            let mut tile_color = if chose_cursor == index && selected.is_some() {
                style::Color::Rgb{r: 0x51, g: 0xb8, b: 0xc6}
            } else if cursor == index {
                style::Color::Rgb{r: 0x21, g: 0x88, b: 0x96}
            } else if is_odd {
                style::Color::Rgb{r: 0x5b, g: 0x3a, b: 0x21}
            } else {
                style::Color::Rgb{r: 0x85, g: 0x5E, b: 0x42}
            };

            
            for move_index in 0..selected_move_length {
                let action = selected_moves[move_index as usize];
                if action.get_target() as usize == index {
                    tile_color = if action.is_capture() && action.is_promotion() {
                        style::Color::Rgb{r: 0xc1, g: 0x8a, b: 0x13}
                    } else if action.is_capture() {
                        if chose_cursor == index {
                            style::Color::Rgb{r: 0xd5, g: 0x74, b: 0x47}
                        } else {
                            style::Color::Rgb{r: 0xa5, g: 0x44, b: 0x17}
                        }
                    } else if action.is_promotion() {
                        style::Color::Rgb{r: 0xad, g: 0xa5, b: 0x14}
                    } else {
                        if chose_cursor == index {
                            style::Color::Rgb{r: 0x76, g: 0xbc, b: 0x45}
                        } else {
                            style::Color::Rgb{r: 0x46, g: 0x8c, b: 0x15}
                        }
                    };
                    break;
                }
            }

            let symbol = if tile.is_empty() {
                "   "
            } else {
                let (piece, _ , _) = game.get_player(tile.get_color()).pieces[tile.get_index() as usize];
                match (piece, tile.get_color()) {
                    (PieceType::None, _) => if tile.is_en_passant() { " + " } else { "   " },
                    
                    (PieceType::Pawn, Color::White) => " ♙ ",
                    (PieceType::Rook, Color::White) => " ♖ ",
                    (PieceType::Knight, Color::White) => " ♘ ",
                    (PieceType::Bishop, Color::White) => " ♗ ",
                    (PieceType::Queen, Color::White) => " ♕ ",
                    (PieceType::King, Color::White) => " ♔ ",
                    
                    (PieceType::Pawn, Color::Black) => " ♟︎ ",
                    (PieceType::Rook, Color::Black) => " ♜ ",
                    (PieceType::Knight, Color::Black) => " ♞ ",
                    (PieceType::Bishop, Color::Black) => " ♝ ",
                    (PieceType::Queen, Color::Black) => " ♛ ",
                    (PieceType::King, Color::Black) => " ♚ ",
                    _ =>  "   ",
                }
            };
            

            /*
            let symbol = format!(" {} ", tile.get_index());
            let tile_color = if tile.is_empty() {
                style::Color::Rgb{r: 0x46, g: 0xff, b: 0x15}
            } else {
                style::Color::Rgb{r: 0xff, g: 0x8c, b: 0x15}
            };
             */

            execute!(
                buffer,           
                style::SetForegroundColor(match tile.get_color() {
                    Color::White => style::Color::White,
                    Color::Black => style::Color::Black,
                }),
                style::SetBackgroundColor(tile_color),
                style::Print(symbol),

                style::ResetColor,
            )?;
        }
        execute!(
            buffer,
            style::Print("\n")
        )?;
    }
    if let Ok(result) = String::from_utf8(buffer) {
        execute!(
            std::io::stdout(),
            style::Print(result),
        )?;
    }

    Ok(())
}
