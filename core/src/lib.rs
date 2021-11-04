#![feature(total_cmp)]
#![feature(try_trait_v2)]
#![feature(never_type)]

pub mod bitboard;


use bitboard::{
    PlayerBitboard,
    Bitboard
};
use nanoserde::{SerJson, DeJson};

use std::{fmt::{self, Debug, write}, io::{BufRead, Write}};
use nanorand::{Rng, WyRand};
use rayon::prelude::*;



use arrayvec::{ArrayString, ArrayVec};





#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum MoveType {
    Quiet          = 0b00,
    PawnDoubleMove  = 0b01,
    KingCastle      = 0b10,
    QueenCastle     = 0b11,
    Capture             = Self::CAPTURE_FLAG | 0b00,
    CaptureEnPassant    = Self::CAPTURE_FLAG | 0b01,
    PromotionQueen      = Self::PROMOTION_FLAG | 0b00,
    PromotionKnight     = Self::PROMOTION_FLAG | 0b01,
    PromotionRook       = Self::PROMOTION_FLAG | 0b10,
    PromotionBishop     = Self::PROMOTION_FLAG | 0b11,
    CapturePromotionQueen   = Self::CAPTURE_FLAG | Self::PROMOTION_FLAG | 0b00,
    CapturePromotionKnight  = Self::CAPTURE_FLAG | Self::PROMOTION_FLAG | 0b01,
    CapturePromotionRook    = Self::CAPTURE_FLAG | Self::PROMOTION_FLAG | 0b10,
    CapturePromotionBishop  = Self::CAPTURE_FLAG | Self::PROMOTION_FLAG | 0b11,
}
impl MoveType {
    const SIZE: u8 = 4;
    const PROMOTION_FLAG: u8    = 0b1000;
    const CAPTURE_FLAG: u8      = 0b0100;
    
    const A: u8 = MoveType::Quiet as u8;
    const B: u8 = MoveType::PawnDoubleMove as u8;
    const C: u8 = MoveType::KingCastle as u8;
    const D: u8 = MoveType::QueenCastle as u8;
    const E: u8 = MoveType::Capture as u8;
    const F: u8 = MoveType::CaptureEnPassant as u8;
    const G: u8 = MoveType::PromotionQueen as u8;
    const H: u8 = MoveType::PromotionKnight as u8;
    const I: u8 = MoveType::PromotionRook as u8;
    const J: u8 = MoveType::PromotionBishop as u8;
    const K: u8 = MoveType::CapturePromotionQueen as u8;
    const L: u8 = MoveType::CapturePromotionKnight as u8;
    const M: u8 = MoveType::CapturePromotionRook as u8;
    const N: u8 = MoveType::CapturePromotionBishop as u8;
}
impl TryFrom<u8> for MoveType {
    type Error = ();
    fn try_from(item: u8) -> Result<Self, Self::Error> {
        match item {
            Self::A => Ok(Self::Quiet),
            Self::B => Ok(Self::PawnDoubleMove),
            Self::C => Ok(Self::KingCastle),
            Self::D => Ok(Self::QueenCastle),
            Self::E => Ok(Self::Capture),
            Self::F => Ok(Self::CaptureEnPassant),
            Self::G => Ok(Self::PromotionQueen),
            Self::H => Ok(Self::PromotionKnight),
            Self::I => Ok(Self::PromotionRook),
            Self::J => Ok(Self::PromotionBishop),
            Self::K => Ok(Self::CapturePromotionQueen),
            Self::L => Ok(Self::CapturePromotionKnight),
            Self::M => Ok(Self::CapturePromotionRook),
            Self::N => Ok(Self::CapturePromotionBishop),
            _ => Err(()),
        }
    }
}
impl Into<u8> for MoveType {
    fn into(self) -> u8 {
        match self {
            MoveType::Quiet => Self::A,
            MoveType::PawnDoubleMove => Self::B,
            MoveType::KingCastle => Self::C,
            MoveType::QueenCastle => Self::D,
            MoveType::Capture => Self::E,
            MoveType::CaptureEnPassant => Self::F,
            MoveType::PromotionQueen => Self::G,
            MoveType::PromotionKnight => Self::H,
            MoveType::PromotionRook => Self::I,
            MoveType::PromotionBishop => Self::J,
            MoveType::CapturePromotionQueen => Self::K,
            MoveType::CapturePromotionKnight => Self::L,
            MoveType::CapturePromotionRook => Self::M,
            MoveType::CapturePromotionBishop => Self::N,
        }
    }
}


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Move (u16);
impl Move {
    const INDEX_SIZE: u8    = 6;
    const TYPE_MASK: u16    = 0b01111;

    const TARGET_OFFSET: u16    = MoveType::SIZE as u16;
    const TARGET_MASK: u16      = 0b111111 << Self::TARGET_OFFSET;
    
    const FROM_OFFSET: u16    = (MoveType::SIZE + Self::INDEX_SIZE) as u16;
    const FROM_MASK: u16      = 0b111111 << Self::FROM_OFFSET;

}
impl Move {
    pub fn new(move_type: MoveType, from: u8, target: u8) -> Self {
        let type_bits = <MoveType as Into<u8>>::into(move_type) as u16 & Self::TYPE_MASK;
        let target_bits = ((target as u16) << Self::TARGET_OFFSET) & Self::TARGET_MASK;
        let from_bits = ((from as u16) << Self::FROM_OFFSET) & Self::FROM_MASK;
        Move(target_bits | from_bits | type_bits)
    }

    pub fn is_promotion(self) -> bool {
        (self.0 as u8 & MoveType::PROMOTION_FLAG) == MoveType::PROMOTION_FLAG
    }
    pub fn is_capture(self) -> bool {
        (self.0 as u8 & MoveType::CAPTURE_FLAG) == MoveType::CAPTURE_FLAG
    }
    pub fn get_target(self) -> u8 {
        ((self.0 & Self::TARGET_MASK) >> Self::TARGET_OFFSET) as u8
    }
    pub fn get_from(self) -> u8 {
        ((self.0 & Self::FROM_MASK) >> Self::FROM_OFFSET) as u8
    }
    pub fn get_type(self) -> MoveType {
        let flags = (self.0 & Self::TYPE_MASK) as u8;
        MoveType::try_from(flags).expect("wtf")
    }
}
impl Default for Move {
    fn default() -> Self {
        Self(0)
    }
}
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({})", self.get_type(), self.get_target())
    }
}
impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({})", self.get_type(), self.get_target())
    }
}





#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0b0,
    White = 0b1,
}
impl Color {
    const VALUE_BLACK: u8 = Self::Black as u8;
    const VALUE_WHITE: u8 = Self::White as u8;
}
impl TryFrom<u8> for Color {
    type Error = ();
    fn try_from(item: u8) -> Result<Self, Self::Error> {
        match item {
            Self::VALUE_BLACK => Ok(Self::Black),
            Self::VALUE_WHITE => Ok(Self::White),
            _ => Err(()),
        }
    }
}
impl Into<u8> for Color {
    fn into(self) -> u8 {
        match self {
            Self::Black => Self::VALUE_BLACK,
            Self::White => Self::VALUE_WHITE,
        }
    }
}
impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}




#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PieceType {
    None = 0,

    Pawn = 0b001,
    Rook = 0b010,
    Knight = 0b011,
    Bishop = 0b100,
    Queen = 0b101,
    King = 0b110,
}
impl PieceType {
    const VALUE_NONE: u8 = Self::None as u8;
    const VALUE_PAWN: u8 = Self::Pawn as u8;
    const VALUE_ROOK: u8 = Self::Rook as u8;
    const VALUE_KNIGHT: u8 = Self::Knight as u8;
    const VALUE_BISHOP: u8 = Self::Bishop as u8;
    const VALUE_QUEEN: u8 = Self::Queen as u8;
    const VALUE_KING: u8 = Self::King as u8;
}
impl PieceType {
    pub fn value(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Pawn => 100.0,
            Self::Knight => 300.0,
            Self::Rook => 400.0,
            Self::Bishop => 500.0,
            Self::Queen => 900.0,
            Self::King => 999999.0,
        }   
    }
}
impl TryFrom<u8> for PieceType {
    type Error = ();
    fn try_from(item: u8) -> Result<Self, Self::Error> {
        match item {
            Self::VALUE_PAWN => Ok(Self::Pawn),
            Self::VALUE_ROOK => Ok(Self::Rook),
            Self::VALUE_KNIGHT => Ok(Self::Knight),
            Self::VALUE_BISHOP => Ok(Self::Bishop),
            Self::VALUE_QUEEN => Ok(Self::Queen),
            Self::VALUE_KING => Ok(Self::King),
            _ => Err(()),
        }
    }
}
impl Into<u8> for PieceType {
    fn into(self) -> u8 {
        match self {
            Self::None => Self::VALUE_NONE,
            Self::Pawn => Self::VALUE_PAWN,
            Self::Rook => Self::VALUE_ROOK,
            Self::Knight => Self::VALUE_KNIGHT,
            Self::Bishop => Self::VALUE_BISHOP,
            Self::Queen => Self::VALUE_QUEEN,
            Self::King => Self::VALUE_KING
        }
    }
}
impl Default for PieceType {
    fn default() -> Self {
        Self::None
    }
}


// TODO: make the tile 16it and give it direct acces to piece data to avoid cache misses
// Less indirection makes me and the cpu happy

// 16bit - from least significant bit
// 0, 0: occupied flag
// 1, 5: index of the piece in a players piece array  
// 6, 6: the color of the piece if occupied
// 7, 7: en passant available
// 8, 8: unused
#[derive(Clone, Copy, Debug)]
pub struct Tile(u8);
impl Tile {
    const OCCUPIED_FLAG_MASK: u8    = 0b1       << Self::OCCUPIED_FLAG_OFFSET;
    const PIECE_INDEX_MASK: u8      = 0b1111    << Self::PIECE_INDEX_OFFSET;
    const COLOR_MASK: u8            = 0b1       << Self::COLOR_OFFSET;
    const EN_PASSANT_FLAG_MASK: u8  = 0b1       << Self::EN_PASSANT_FLAG_OFFSET;

    const OCCUPIED_FLAG_OFFSET: u8      = 0;
    const PIECE_INDEX_OFFSET: u8        = Self::OCCUPIED_FLAG_OFFSET    + 1;
    const COLOR_OFFSET: u8              = Self::PIECE_INDEX_OFFSET      + 4;
    const EN_PASSANT_FLAG_OFFSET: u8    = Self::COLOR_OFFSET            + 1;
}
impl Tile {
    pub fn empty() -> Self {
        Self(0)
    }
    pub fn with_piece(index: u8, color: Color) -> Self {
        let color: u8 = color.into();

        let mut result = Self::empty();
        result.0 |= 0b1 << Self::OCCUPIED_FLAG_OFFSET;
        result.0 |= (index << Self::PIECE_INDEX_OFFSET) & Self::PIECE_INDEX_MASK;
        result.0 |= (color << Self::COLOR_OFFSET) & Self::COLOR_MASK;
        result
    }


    pub fn set_en_passant(&mut self, state: bool) {
        if state {
            self.0 |= Self::EN_PASSANT_FLAG_MASK
        } else {
            self.0 &= !Self::EN_PASSANT_FLAG_MASK
        }
    }


    pub fn get_index(&self) -> u8 {
        (self.0 & Self::PIECE_INDEX_MASK) >> Self::PIECE_INDEX_OFFSET
    }
    pub fn get_color(&self) -> Color {
        ((self.0 & Self::COLOR_MASK) >> Self::COLOR_OFFSET)
            .try_into()
            .unwrap()
    }

    pub fn is_occupied(&self) -> bool {
        (self.0 & Self::OCCUPIED_FLAG_MASK) != 0
    }
    pub fn is_empty(&self) -> bool {
        !self.is_occupied()
    }
    pub fn is_color(&self, color: Color) -> bool {
        self.get_color() == color
    }
    pub fn is_en_passant(&self) -> bool {
        (self.0 & Self::EN_PASSANT_FLAG_MASK) != 0
    }
}




#[derive(SerJson, DeJson, Debug)]
pub struct DebugJSON {
    immediate_value: f32,
    value: f32,
    alpha: f32,
    beta: f32,
    map: String,
    children: Vec<DebugJSON>,
}

/*
pub fn ai_play2(game: &mut GameState, depth: usize, color: Color) {
    let mut best_moves = Vec::new();
    let mut best_value = None;

    let pieces = match color {
        Color::White => &game.white.pieces,
        Color::Black => &game.black.pieces,
    };

    let mut alpha = -999999.9;
    let mut beta = 999999.9;

    let mut debug = DebugJSON {
        children: Vec::new(),
        map: game.to_fen(),
        immediate_value: game.value(),
        alpha: alpha,
        beta: beta,
        value: 0.0,
    };

    for (_, moves) in pieces {
        for action in moves {
            let mut game = game.clone();
            game.play_move(color, Position::from_index(action.get_from()), Position::from_index(action.get_target()));
            game.current = !game.current;
            game.generate_moves(Color::Black);
            game.generate_moves(Color::White);
            game.validate_moves();

            let mut next_debug = DebugJSON {
                children: Vec::new(),
                map: game.to_fen(),
                immediate_value: game.value(),
                alpha: alpha,
                beta: beta,
                value: 0.0,
            };

            let value = minimax(game, depth - 1, alpha, beta, !color, &mut next_debug);
            match color {
                Color::White => alpha = alpha.max(value),
                Color::Black => beta = beta.min(value),
            }
            
            if let Some(best) = best_value {
                if value == best {
                    best_moves.push(action.clone());
                } else if value > best && color == Color::White {
                    best_moves.clear();
                    best_moves.push(action.clone());
                    best_value = Some(value);
                } else if value < best && color == Color::Black {
                    best_moves.clear();
                    best_moves.push(action.clone());
                    best_value = Some(value);
                }

            } else {
                best_moves.push(action.clone());
                best_value = Some(value);
            };

            next_debug.value = value;
            next_debug.alpha = alpha;
            next_debug.beta = beta;
            debug.children.push(next_debug);

            if beta <= alpha {
                break;
            }
        }
    }
    //debug.children.sort_by(|a, b| {
    //    a.value.total_cmp(&b.value)
    //});

    let mut rng = nanorand::WyRand::new();
    let value = best_value.unwrap();
    let action = best_moves.get(0).unwrap();
    let action = best_moves[rng.generate_range(0..best_moves.len())];
    game.play_move(color, Position::from_index(action.get_from()), Position::from_index(action.get_target()));
    debug.value = value;


    let mut file = std::fs::File::create("./debug.json").unwrap();
    let mut buffer = debug.serialize_json();
    file.write_all(buffer.as_bytes());
}

pub fn minimax(game: GameState, depth: usize, mut alpha: f32, mut beta: f32, color: Color, debug: &mut DebugJSON) -> f32 {
    if depth == 0 {
        let current_value = game.value();
        return current_value;
    }

    match color {
        Color::White => {
            let mut max_value = f32::NEG_INFINITY;
            'outer: for (_, moves) in &game.white.pieces {
                for child in moves {
                    let mut game = game.clone();

                    game.play_move(color, Position::from_index(child.get_from()), Position::from_index(child.get_target()));
                    game.current = !game.current;
                    for tile in &mut game.board {
                        tile.set_white_attacking(false);
                        tile.set_black_attacking(false);
                    }
                    game.generate_moves(Color::Black);
                    game.generate_moves(Color::White);
                    game.validate_moves();

                    let mut next_debug = DebugJSON {
                        children: Vec::new(),
                        map: game.to_fen(),
                        immediate_value: game.value(),
                        alpha: 0.0,
                        beta: 0.0,
                        value: 0.0,
                    };

                    let next_value = minimax(game, depth - 1, alpha, beta, !color, &mut next_debug);
                    max_value = max_value.max(next_value);
                    alpha = alpha.max(next_value);

                    next_debug.value = next_value;
                    next_debug.alpha = alpha;
                    next_debug.beta = beta;
                    debug.children.push(next_debug);

                    if beta <= alpha {
                        break 'outer;
                    }
                }
            }
            //debug.children.sort_by(|a, b| {
            //    a.value.total_cmp(&b.value)
            //});
            return max_value;
        }
        Color::Black => {
            let mut min_value = f32::INFINITY;
            'outer: for (_, moves) in &game.black.pieces {
                for child in moves {
                    let mut game = game.clone();

                    game.play_move(color, Position::from_index(child.get_from()), Position::from_index(child.get_target()));
                    game.current = !game.current;
                    for tile in &mut game.board {
                        tile.set_white_attacking(false);
                        tile.set_black_attacking(false);
                    }
                    game.generate_moves(Color::Black);
                    game.generate_moves(Color::White);
                    game.validate_moves();

                    let mut next_debug = DebugJSON {
                        children: Vec::new(),
                        map: game.to_fen(),
                        immediate_value: game.value(),
                        alpha: 0.0,
                        beta: 0.0,
                        value: 0.0,
                    };

                    let next_value = minimax(game, depth - 1, alpha, beta, !color, &mut next_debug);
                    min_value = min_value.min(next_value);
                    beta = beta.min(next_value);
                    
                    next_debug.value = next_value;
                    next_debug.alpha = alpha;
                    next_debug.beta = beta;
                    debug.children.push(next_debug);

                    if beta <= alpha {
                        break 'outer;
                    }
                }
            }
            //debug.children.sort_by(|a, b| {
            //    a.value.total_cmp(&b.value)
            //});
            return min_value;
        }
    }
}


pub fn ai_play(game: &mut GameState) -> (bool, bool) {
    let mut rng = WyRand::new();

    let pieces = match game.current {
        Color::White => game.white.pieces.iter(),
        Color::Black => game.black.pieces.iter(),
    };

    let total_moves = pieces
        .map(|(_, moves)| moves.clone().to_vec())
        .flatten()
        .collect::<Vec<Move>>();
    if total_moves.len() == 0 {
        return (false, false);
    }

    let depth = 3;

    use std::sync::{
        Arc,
        Mutex,
        atomic::AtomicIsize,
        atomic::Ordering,
    };


    let legal_moves: Arc<Mutex<Vec<(isize, Move)>>> = Arc::new(Mutex::new(Vec::new()));
    total_moves.par_iter().for_each(|action| {
        let mut game = game.clone();
        
        let position = action.get_target() as usize;
        let (piece, _) = game.board[position].get_piece();
        let mut worth = match piece {
            PieceType::None => 0,
            PieceType::Pawn => 1000,
            PieceType::Knight => 300,
            PieceType::Bishop => 300,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => return,
        };
        worth += total_moves.len() as isize * 1;



        let self_checked = game.is_king_checked(game.current);
        let other_checked = game.is_king_checked(!game.current);
        game.play_move(game.current, Position::from_index(action.get_from()), Position::from_index(action.get_target()));
        if !self_checked && game.is_king_checked(game.current) {
            // Piece is pinned, move is illegal
            // Low mobility
            return;
        }
        if self_checked {
            if game.is_king_checked(game.current) {
                return;
            } else {
                // Remove check, has to be done.
            }
        }
        if !other_checked && game.is_king_checked(!game.current) {
            // Piece kan check king pog.
            worth += 200;
        }

        game.generate_moves(game.current);
        game.generate_moves(!game.current);
        game.validate_moves();
        game.current = !game.current;

        let opponent = match game.current {
            Color::White => game.white.pieces.iter(),
            Color::Black => game.black.pieces.iter(),
        };
    
        let opponent_moves = opponent
            .map(|(_, moves)| moves.clone().to_vec())
            .flatten()
            .collect::<Vec<Move>>();

        let score = worth - evaluate(&game, depth, opponent_moves, game.current) / depth as isize;
        match legal_moves.lock().as_mut() {
            Ok(moves) => {
                moves.push((score, action.clone()));
            }
            _ => (),
        }
    });


    let moves: Vec<(isize, Move)> = match legal_moves.lock().as_mut() {
        Ok(moves) => {
            if moves.len() == 0 {
                return (false, false); // ???
            }
            moves.sort_by(|(a, _), (b, _)| { b.cmp(a) });0;
            let lowest = moves[0].0;
            moves.iter().filter(|(score, _)| { *score == lowest }).map(|a| *a ).collect()
        }
        _ => return (false, false),
    };



    let action = moves[rng.generate_range(0..moves.len())].1;
    (game.play_move(game.current, Position::from_index(action.get_from()), Position::from_index(action.get_target())), action.is_capture())
}

pub fn evaluate(game: &GameState, depth: usize, moves: Vec<Move>, color: Color) -> isize {
    if depth == 0 || moves.len() == 0{
        return 0;
    }

    // Mobility score
    let mut total = moves.len() as isize * 1;

    let mut alpha = -10000;

    for action in &moves {
        let mut game = game.clone();

        let position = action.get_target() as usize;
        let (piece, _) = game.board[position].get_piece();
        let mut worth = match piece {
            PieceType::None => 0,
            PieceType::Pawn => 100,
            PieceType::Knight => 300,
            PieceType::Bishop => 300,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => {
                total += 200;
                continue;
            }
        };
        

        let self_checked = game.is_king_checked(color);
        let other_checked = game.is_king_checked(!color);
        game.play_move(color, Position::from_index(action.get_from()), Position::from_index(action.get_target()));
        if !self_checked && game.is_king_checked(color) {
            // Piece is pinned, move is illegal
            continue;
        }
        if self_checked {
            if game.is_king_checked(color) {
                continue;
            } else {
                // Remove check, has to be done.
            }
        }
        
        if !other_checked && game.is_king_checked(!color) {
            // Piece kan check king pog.
            worth += 99;
        }

        // Alpha pruning ?????
        if worth < alpha {
            continue;
        } else if worth > alpha { 
            alpha = worth;
        }

        game.generate_moves(color);
        game.generate_moves(!color);
        game.validate_moves();
        game.current = !game.current;



        let opponent = match game.current {
            Color::White => game.white.pieces.iter(),
            Color::Black => game.black.pieces.iter(),
        };
    
        let total_moves = opponent
            .map(|(_, moves)| moves.clone().to_vec())
            .flatten()
            .collect::<Vec<Move>>();

        // Check opponents response
        worth -= evaluate(&game, depth - 1, total_moves, !color);
        total += worth;
    }

    total / moves.len() as isize
}

pub fn material(game: &GameState, action: &Move) -> isize {
    let position = action.get_target() as usize;
    let (piece, _) = game.board[position].get_piece();
    match piece {
        PieceType::None => 0,
        PieceType::Pawn => 100,
        PieceType::Knight => 300,
        PieceType::Rook => 400,
        PieceType::Bishop => 500,
        PieceType::Queen => 900,
        PieceType::King => 999999,
    }
}
*/









#[derive(Clone, Default, Debug)]
pub struct PlayerState {
    pub pieces: ArrayVec<Piece, 16>,
    pub bitboard: PlayerBitboard,
    pub can_queen_castle: bool,
    pub can_king_castle: bool,
}
impl PlayerState {
    pub fn value(&self) -> f32 {
        let mut value = 0.0;
        value += self.pieces.iter().fold(0.0, |value, (piece, _, _)| {
            value + piece.value()
        });
        value
    }
    pub fn king(&self) -> &Piece {
        &self.pieces[0]
    }
    pub fn king_mut(&mut self) -> &mut Piece {
        &mut self.pieces[0]
    }
}



pub enum Direction {
    North,
    NorthEast,
    NorthWest,

    South, 
    SouthEast, 
    SouthWest, 

    East, 
    West,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Coord {
    pub file: u8,
    pub rank: u8,
}
impl Coord {
    pub fn new(file: u8, rank: u8) -> Self {
        Self {
            file,
            rank,
        }
    }
    pub fn from(index: u8) -> Self {
        Self::new(index % 8, index / 8)
    }

    pub fn index(self) -> Option<usize> {
        if self.file >= 8 || self.rank >= 8 {
            None
        } else {
            Some((self.rank * 8 + self.file) as usize)
        }
    }
    pub fn offset(self, file: i8, rank: i8) -> Self {
        Self {
            file: (self.file as i8 + file) as u8,
            rank: (self.rank as i8 + rank) as u8,
        }
    }
}
impl std::ops::Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            file: self.file + rhs.file,
            rank: self.rank + rhs.rank,
        }
    }
}
impl std::ops::Sub for Coord {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            file: self.file - rhs.file,
            rank: self.rank - rhs.rank,
        }
    }
}




pub enum GameStateEndTerm {
    None,
    Stalemate,
    Checkmate,
    DeadPosition,
    ThreeFoldRepetition,
    Move50Rule,
}


// A peice can max hold 28 moves i think
// TODO verify this, currently i use 32 for buffer spcaes
type PieceMoves = ArrayVec<Move, 32>;
// TODO: make this an actual named struct
type Piece = (PieceType, Coord, PieceMoves);


pub enum MoveSpecialRules {
    PownMove,
    PawnAttack,
    PawnDoublePush,
}


pub type Board = [Tile; 64];


#[derive(Clone, Debug)]
pub struct GameState {
    pub board: Board,
    pub current: Color,
    pub white: PlayerState,
    pub black: PlayerState,

    pub en_passant: Option<Coord>,

    pub halfmove_count: u32,
    pub fullmove_count: u32,
}

impl GameState {
    // move gen
    pub fn generate_pseudo_legal_moves(&mut self) {

    }
    pub fn generate_legal_moves(&mut self) {

    }
}

impl GameState {
    pub fn get_player(&self, color: Color) -> &PlayerState {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
    pub fn get_player_mut(&mut self, color: Color) -> &mut PlayerState {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    pub fn value(&self) -> f32 {
        self.white.value() - self.black.value()
    }


    pub fn play_move(&mut self, movement: Move) -> bool {
        false
    }
    pub fn play_move_unchecked(&mut self, movement: Move) {
        let to = movement.get_target();
        let from = movement.get_from();
        let attacker = self.board[from as usize];
        let target = self.board[to as usize];

        // Update board state
        self.board[to as usize] = attacker;
        self.board[from as usize] = Tile::empty();

        // Update attacker state
        {
            let index = attacker.get_index() as usize;
            let player = self.get_player_mut(self.current);
            let (attacker_type, coord, _) = &mut player.pieces[index];
    
            // Clear from bit
            player.bitboard.unset_bit(*attacker_type, *coord);
            // Set corrd to the new position
            *coord = Coord::from(to);
            // Set to bit
            player.bitboard.set_bit(*attacker_type, *coord);
        }

        // Update target state if neccesary
        if target.is_occupied() {
            let index = target.get_index() as usize;
            let enemy = self.get_player_mut(!self.current);
            let piece = &mut enemy.pieces[index];
            enemy.bitboard.unset_bit(piece.0, piece.1);
            *piece = (PieceType::None, Coord::default(), ArrayVec::new());
        }
    }


    pub fn generate_king_masks(&self, color: Color) -> (Bitboard, Bitboard) {
        let player = self.get_player(color).bitboard;
        let enemy = self.get_player(!color).bitboard;
        let mut occlusion = player.occupancy() | enemy.king;
        
        // TODO: fleeing does not account for pieces that are right next to the king, that (pieces) could be taken
        // TODO: or the enemy occupancy into the mask - if an enemy attack range does not include itself
        let enemy_attacks = enemy.attacks(occlusion, !color);
        let king_defence_mask = !enemy_attacks | enemy.occupancy();
        if (enemy_attacks & player.king).0 == 0 {
            // Player king is not in check
            return (Bitboard(!0), king_defence_mask);
        }
        
        occlusion |= enemy.occupancy() | player.king;
        let rays_diagonal = PlayerBitboard::generate_rook_attacks(player.king, occlusion);
        let rays_straight = PlayerBitboard::generate_bishop_attacks(player.king, occlusion);
        let rays_all = rays_straight | rays_diagonal;

        let king_check_mask = {
            (rays_diagonal & PlayerBitboard::generate_rook_attacks(enemy.rooks, occlusion)) | 
            (rays_straight & PlayerBitboard::generate_bishop_attacks(enemy.bishops, occlusion)) |
            (rays_all & PlayerBitboard::generate_queen_attacks(enemy.queens, occlusion)) |
            (enemy.knights & PlayerBitboard::generate_knight_attacks(player.king)) |
            (enemy.pawns & PlayerBitboard::generate_pawn_attacks(player.king, color))
        };

        (king_check_mask, king_defence_mask)
    }

    pub fn generate_moves(&mut self, color: Color) {
        let board = self.board.clone();
        let player_bitboard = self.get_player(color).bitboard.clone();
        let enemy_bitboard = self.get_player(!color).bitboard.clone();
        let (king_check_mask, king_defence_mask) = self.generate_king_masks(color);

        let will_move_check_self = |piece_type: PieceType, from: Coord, to: Coord| -> bool {
            // TODO this is temp, use just occupancy instead if that works
            let mut player_bitboard = player_bitboard.clone();
            let mut enemy_bitboard = enemy_bitboard.clone();

            player_bitboard.set_bit(piece_type, to);
            player_bitboard.unset_bit(piece_type, from);
            enemy_bitboard.unset_bit(PieceType::Bishop, to);
            enemy_bitboard.unset_bit(PieceType::Rook, to);
            enemy_bitboard.unset_bit(PieceType::Queen, to);

            let occlusion = player_bitboard.occupancy();
            let enemy_attacks = {
                PlayerBitboard::generate_bishop_attacks(enemy_bitboard.bishops, occlusion) |
                PlayerBitboard::generate_rook_attacks(enemy_bitboard.rooks, occlusion) |
                PlayerBitboard::generate_queen_attacks(enemy_bitboard.queens, occlusion)
            };

            (player_bitboard.king & enemy_attacks).0 != 0
        };


        let single_move_attack = |result: &mut PieceMoves, piece_type: PieceType, from: Coord, to: Coord, movement_mask: Bitboard| -> bool {
            let to_index = match to.index() {
                Some(index) => index as u8,
                None => return true,
            };
            let from_index = match from.index() {
                Some(index) => index as u8,
                None => return true,
            };

            if !movement_mask.is_occupied(to_index as usize) {
                return false;
            }
            if will_move_check_self(piece_type, from, to) {
                return false;
            }
            

            let tile = board[to_index as usize];
            if tile.is_empty() {
                // quiet move
                result.push(Move::new(MoveType::Quiet, from_index, to_index));
                false
            } else if color != tile.get_color() {
                // capture
                result.push(Move::new(MoveType::Capture, from_index, to_index));
                true
            } else {
                // Blocked
                true
            }
        };
        let sliding_moves = |mut result: PieceMoves, piece_type: PieceType, from: Coord, direction: Direction, movement_mask: Bitboard| -> PieceMoves {
            let (dx, dy ) = match direction {
                Direction::North        => ( 0, -1),
                Direction::NorthEast    => ( 1, -1),
                Direction::NorthWest    => (-1, -1),
                Direction::South        => ( 0,  1),
                Direction::SouthEast    => ( 1,  1),
                Direction::SouthWest    => (-1,  1),
                Direction::East         => ( 1,  0),
                Direction::West         => (-1,  0),
            };
    
            for distance in 1..(8 as i8) {
                let to = from.offset(dx * distance, dy * distance);
                if single_move_attack(&mut result, piece_type, from, to, movement_mask) {
                    break;
                }
            }
            result
        };
        let pawn_moves = |mut result: PieceMoves, from: Coord| -> PieceMoves {
            let from_index = match from.index() {
                Some(index) => index as u8,
                None => return result,
            };


            let capture_diagonal = |result: &mut PieceMoves, to: Coord, promotion_rank: u8| {
                let to_index = match to.index() {
                    Some(index) => index as u8,
                    None => return,
                };

                let tile = board[to_index as usize];
                if tile.is_occupied() && color != tile.get_color() {
                    if to.rank == promotion_rank {
                        result.push(Move::new(MoveType::CapturePromotionKnight, from_index, to_index));
                        result.push(Move::new(MoveType::CapturePromotionBishop, from_index, to_index));
                        result.push(Move::new(MoveType::CapturePromotionRook, from_index, to_index));
                        result.push(Move::new(MoveType::CapturePromotionQueen, from_index, to_index));
                    } else {
                        result.push(Move::new(MoveType::Capture, from_index, to_index));
                    }
                }
            };
            let move_forward = |result: &mut PieceMoves, to: Coord, promotion_rank: u8| -> bool {
                let to_index = match to.index() {
                    Some(index) => index as u8,
                    None => return false,
                };
                
                let tile = board[to_index as usize];
                if tile.is_empty() {
                    if to.rank == promotion_rank {
                        result.push(Move::new(MoveType::PromotionKnight, from_index, to_index));
                        result.push(Move::new(MoveType::PromotionBishop, from_index, to_index));
                        result.push(Move::new(MoveType::PromotionRook, from_index, to_index));
                        result.push(Move::new(MoveType::PromotionQueen, from_index, to_index));
                    } else {
                        result.push(Move::new(MoveType::Quiet, from_index, to_index));
                    }
                    true
                } else {
                    false
                }
            };

            let (dy, push_rank, promotion_rank) = match color {
                Color::White => (-1, 6, 0),
                Color::Black => (1, 1, 7),
            };
            capture_diagonal(&mut result, from.offset(1, dy), promotion_rank);
            capture_diagonal(&mut result, from.offset(-1, dy), promotion_rank);
            let is_empty = move_forward(&mut result, from.offset(0, dy), promotion_rank);
            if from.rank == push_rank && is_empty {
                move_forward(&mut result, from.offset(0, 2 * dy), promotion_rank);
            }
            result
        };
        /*
        let mut king_moves = || {
            let player = self.get_player(color);
            let enemy_attacks = self.get_player(!color).bitboard.attacks(player.bitboard.occupancy(), color);
    
            let player = self.get_player_mut(color);
            let (_, king_coord, king_moves) = &mut player.king;
            for index in 0..8 {
                index
            }
            single_move_attack(king_moves, *king_coord, king_coord.offset( 1, 1));
            single_move_attack(king_moves, *king_coord, king_coord.offset( 0, 1));
            single_move_attack(king_moves, *king_coord, king_coord.offset( -1, 1));
            single_move_attack(king_moves, *king_coord, king_coord.offset( 1, 0));
            single_move_attack(king_moves, *king_coord, king_coord.offset( -1, 0));
            single_move_attack(king_moves, *king_coord, king_coord.offset( 1, -1));
            single_move_attack(king_moves, *king_coord, king_coord.offset( 0, -1));
            single_move_attack(king_moves, *king_coord, king_coord.offset( -1, -1));
        };

        king_moves();
        */

        for (piece, coord, moves) in &mut self.get_player_mut(color).pieces {
            let coord = *coord;
            moves.clear();

            let mut result: PieceMoves = ArrayVec::new();
            match piece {
                PieceType::Pawn => {
                    result = pawn_moves(result, coord);
                }
                PieceType::Rook => {
                    result = sliding_moves(result, PieceType::Rook, coord, Direction::North, king_check_mask);
                    result = sliding_moves(result, PieceType::Rook, coord, Direction::South, king_check_mask);
                    result = sliding_moves(result, PieceType::Rook, coord, Direction::East, king_check_mask);
                    result = sliding_moves(result, PieceType::Rook, coord, Direction::West, king_check_mask);
                }
                PieceType::Bishop => {
                    result = sliding_moves(result, PieceType::Bishop, coord, Direction::NorthEast, king_check_mask);
                    result = sliding_moves(result, PieceType::Bishop, coord, Direction::NorthWest, king_check_mask);
                    result = sliding_moves(result, PieceType::Bishop, coord, Direction::SouthEast, king_check_mask);
                    result = sliding_moves(result, PieceType::Bishop, coord, Direction::SouthWest, king_check_mask);
                }
                PieceType::Knight => {
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( 2, 1), king_check_mask);
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( 2, -1), king_check_mask);
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( -2, 1), king_check_mask);
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( -2, -1), king_check_mask);
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( 1, 2), king_check_mask);
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( 1, -2), king_check_mask);
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( -1, 2), king_check_mask);
                    single_move_attack(&mut result, PieceType::Knight, coord, coord.offset( -1, -2), king_check_mask);
                }
                PieceType::Queen => {
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::North, king_check_mask);
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::South, king_check_mask);
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::East, king_check_mask);
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::West, king_check_mask);
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::NorthEast, king_check_mask);
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::NorthWest, king_check_mask);
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::SouthEast, king_check_mask);
                    result = sliding_moves(result, PieceType::Queen, coord, Direction::SouthWest, king_check_mask);
                }
                PieceType::King => {
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( 1, 1), king_defence_mask);
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( 1, 0), king_defence_mask);
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( 1, -1), king_defence_mask);
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( 0, 1), king_defence_mask);
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( 0, -1), king_defence_mask);
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( -1, 1), king_defence_mask);
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( -1, 0), king_defence_mask);
                    single_move_attack(&mut result, PieceType::King, coord, coord.offset( -1, -1), king_defence_mask);
                }
                _ => (),
            }

            *moves = result;
        }
    }
    // TODO: rename to filter pseudo legal moves
    // TODO: rename previous functions to generate_pseudo_legal_moves and this one to generate_legal_moves
    // And remove the tile is attacking check from the generating code, i can only know whos attacking what after generation aka here
    /*
    pub fn validate_moves(&mut self) {
        let mut game = self.clone();

        // Current move can't put king in check
        for (piece, _, moves) in &mut self.white.pieces {
            if *piece == PieceType::King {
                *moves = moves.iter().filter(|&action| {
                    let tile = self.board.get_mut(action.get_target() as usize).unwrap();
                    if game.board[action.get_target() as usize].is_black_attacking() {
                        tile.set_white_attacking(false);
                        false
                    } else {
                        true
                    }
                }).map(|action| *action).collect();
            }
        }
        for (piece, _, moves) in &mut self.black.pieces {
            if *piece == PieceType::King {
                *moves = moves.iter().filter(|&action| {
                    let tile = self.board.get_mut(action.get_target() as usize).unwrap();
                    if game.board[action.get_target() as usize].is_white_attacking() {
                        tile.set_black_attacking(false);
                        false
                    } else {
                        true
                    }
                }).map(|action| *action).collect();
            }
        }


        // If king is checked, the next move has to free the king
        match self.current {
            Color::White => {
                for (_, _, moves) in &mut self.white.pieces {
                    *moves = moves.iter().filter(|&action| {
                        let mut game = game.clone();
                        game.play_move(Color::White, Position::from_index(action.get_from()), Position::from_index(action.get_target()));
                        game.generate_moves(Color::Black);
                        game.generate_moves(Color::White);
        
                        // Filter out any moves that leave king in check
                        !game.is_king_checked(Color::White)
                    }).map(|action| *action).collect();
                }
            }
            Color::Black => {
                for (_, _, moves) in &mut self.black.pieces {
                    *moves = moves.iter().filter(|&action| {
                        let mut game = game.clone();
                        game.play_move(Color::Black, Position::from_index(action.get_from()), Position::from_index(action.get_target()));
                        game.generate_moves(Color::Black);
                        game.generate_moves(Color::White);
        
                        // Filter out any moves that leave king in check
                        !game.is_king_checked(Color::Black)
                    }).map(|action| *action).collect();
                }
            }
        }
    }
    */
    pub fn move_count(&self, color: Color) -> usize {
        let mut move_count = 0;
        let pieces = match color {
            Color::White => &self.white.pieces,
            Color::Black => &self.black.pieces,
        };
        pieces.iter().for_each(|(_, _, moves)| {
            move_count += moves.len();
        });
        move_count
    }
    pub fn is_king_stalemated(&self, color: Color) -> bool {
        self.move_count(color) == 0 && !self.is_king_checked(color)
    }
    pub fn is_king_checkmated(&self, color: Color) -> bool {
        self.move_count(color) == 0 && self.is_king_checked(color)
    }
    pub fn is_king_checked(&self, color: Color) -> bool {
        let player = self.get_player(color);
        let opponent = self.get_player(!color);
        let difference = player.bitboard.king & opponent.bitboard.attacks(player.bitboard.occupancy(), !color);
        difference.0 != 0
    }


    pub fn push_piece(&mut self, piece: PieceType, color: Color, coord: Coord) {
        let index = match color {
            Color::White => {
                self.white.bitboard.set_bit(piece, coord);
                self.white.pieces.push((piece, coord, ArrayVec::new()));
                self.white.pieces.len() - 1
            }
            Color::Black => {
                self.black.bitboard.set_bit(piece, coord);
                self.black.pieces.push((piece, coord, ArrayVec::new()));
                self.black.pieces.len() - 1
            }
        };
        self.board[coord.index().unwrap()] = Tile::with_piece(index as u8, color);
    }


    pub fn to_fen(&self) -> String {
        // + 8 * 8 for board
        // + 7 for '/'
        // + 1 for curent player
        // + 4 for castling rights
        // + 2 for en passant
        // + 3 for whitespace
        const FEN_MAX_LENGTH: usize = (((8 * 8) + 7) + (1 + 4 + 2 )) + 3;
        let mut buffer: ArrayString<FEN_MAX_LENGTH> = ArrayString::new();


        // game board state
        let mut iter = self.board.iter().enumerate().peekable();
        'outer: while let Some(&(index, tile)) = iter.peek() {
            if index % 8 == 0 && index != 0 {
                buffer.push('/');
            }
            
            if tile.is_empty() {
                iter.next();
                while let Some(&(next_index, tile)) = iter.peek() {
                    if !tile.is_empty() || next_index % 8 == 0 && next_index != 0 {
                        buffer.push(char::from_digit((next_index - index) as u32, 10).unwrap());
                        continue 'outer;
                    }
                    iter.next();
                }
            } else {
                let symbol = if tile.is_empty() {
                    ' '
                } else {
                    let tile_color = tile.get_color();
                    let (piece, _ , _) = self.get_player(tile_color).pieces[tile.get_index() as usize];
                    match (piece, tile_color) {
                        (PieceType::Pawn, Color::Black) => 'p',
                        (PieceType::Knight, Color::Black) => 'n',
                        (PieceType::Rook, Color::Black) => 'r',
                        (PieceType::Bishop, Color::Black) => 'b',
                        (PieceType::Queen, Color::Black) => 'q',
                        (PieceType::King, Color::Black) => 'k',
    
                        (PieceType::Pawn, Color::White) => 'P',
                        (PieceType::Knight, Color::White) => 'N',
                        (PieceType::Rook, Color::White) => 'R',
                        (PieceType::Bishop, Color::White) => 'B',
                        (PieceType::Queen, Color::White) => 'Q',
                        (PieceType::King, Color::White) => 'K',
                        _ => ' ',
                    }
                };
                buffer.push(symbol)
            }
            iter.next();
        }
        buffer.push(' ');


        // first to move
        buffer.push(match self.current {
            Color::Black => 'b',
            Color::White => 'w',
        });
        buffer.push(' ');


        // castling rights
        let white_rights = self.white.can_king_castle && self.white.can_queen_castle;
        let black_rights = self.black.can_king_castle && self.black.can_queen_castle;
        if !white_rights && !black_rights {
            buffer.push('-')
        } else {
            if self.white.can_king_castle  { buffer.push('K') };
            if self.white.can_queen_castle { buffer.push('Q') };
            if self.black.can_king_castle  { buffer.push('k') };
            if self.black.can_queen_castle { buffer.push('q') };
        }
        buffer.push(' ');


        // en passant
        buffer.push('-');
        buffer.push(' ');


        // halfmove count
        let mut temp = String::with_capacity(10);
        itoa::fmt(&mut temp, self.halfmove_count);
        buffer.push_str(&temp);
        buffer.push(' ');

        // fullmove count
        let mut temp = String::with_capacity(10);
        itoa::fmt(&mut temp, self.fullmove_count);
        buffer.push_str(&temp);

        
        buffer.to_string()
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut result = Self::default();
        let fields: ArrayVec<&str, 6> = fen.split_whitespace().collect();
    
        
        
        // Parse board positions
        let mut white_king_coord = None;
        let mut black_king_coord = None;
        result.white.pieces.push((PieceType::King, Coord::default(), ArrayVec::new()));
        result.black.pieces.push((PieceType::King, Coord::default(), ArrayVec::new()));
        if let Some(&field) = fields.get(0) {
            let ranks: ArrayVec<&str, 8> = field.split("/").collect();
            if ranks.len() != 8 {
                return None;
            }

            for (rank_index, rank) in ranks.iter().enumerate() {
                let rank_index = rank_index as u8;
                let mut file_index = 0;
                for file in rank.chars() {
                    match file {
                        'p' => result.push_piece(PieceType::Pawn, Color::Black, Coord::new(file_index, rank_index)),
                        'r' => result.push_piece(PieceType::Rook, Color::Black, Coord::new(file_index, rank_index)),
                        'n' => result.push_piece(PieceType::Knight, Color::Black, Coord::new(file_index, rank_index)),
                        'b' => result.push_piece(PieceType::Bishop, Color::Black, Coord::new(file_index, rank_index)),
                        'q' => result.push_piece(PieceType::Queen, Color::Black, Coord::new(file_index, rank_index)),
                        'k' => match black_king_coord {
                            None => {
                                let coord = Coord::new(file_index, rank_index);
                                result.board[coord.index()?] = Tile::with_piece(0, Color::Black);
                                result.black.bitboard.set_bit(PieceType::King, coord);
                                black_king_coord = Some(coord);
                            }
                            Some(_) => return None,
                        },
            
                        'P' => result.push_piece(PieceType::Pawn, Color::White, Coord::new(file_index, rank_index)),
                        'R' => result.push_piece(PieceType::Rook, Color::White, Coord::new(file_index, rank_index)),
                        'N' => result.push_piece(PieceType::Knight, Color::White, Coord::new(file_index, rank_index)),
                        'B' => result.push_piece(PieceType::Bishop, Color::White, Coord::new(file_index, rank_index)),
                        'Q' => result.push_piece(PieceType::Queen, Color::White, Coord::new(file_index, rank_index)),
                        'K' => match white_king_coord {
                            None => {
                                let coord = Coord::new(file_index, rank_index);
                                result.board[coord.index()?] = Tile::with_piece(0, Color::White);
                                result.white.bitboard.set_bit(PieceType::King, coord);
                                white_king_coord = Some(coord)
                            }
                            Some(_) => return None,
                        },
            
                        '1' => file_index += 1 - 1,
                        '2' => file_index += 2 - 1,
                        '3' => file_index += 3 - 1,
                        '4' => file_index += 4 - 1,
                        '5' => file_index += 5 - 1,
                        '6' => file_index += 6 - 1,
                        '7' => file_index += 7 - 1,
                        '8' => file_index += 8 - 1,

                        _ => return None,
                    }

                    file_index += 1;
                }
                // TODO: ensure length of each file is 8
                /*if file_index != 8 {
                    return None;
                }*/
            }
        }
        result.white.pieces[0].1 = white_king_coord?;
        result.black.pieces[0].1 = black_king_coord?;
    
    
        // Parse current player color
        if let Some(&field) = fields.get(1) {
            match field {
                "w" => result.current = Color::White,
                "b" => result.current = Color::Black,
                _ => return None,
            }
        }
    

        if result.is_king_checked(Color::Black) && result.current == Color::White {
            return None;
        }
        if result.is_king_checked(Color::White) && result.current == Color::Black {
            return None;
        }
        
    
        // Parse castling rights
        if let Some(&field) = fields.get(2) {
            if field.len() > 4 {
                return None;
            }
    
            if Some('-') != field.chars().nth(0) {
                for character in field.chars() {
                    match character {
                        'k' => result.black.can_king_castle = true,
                        'q' => result.black.can_queen_castle = true,
                        
                        'K' => result.white.can_king_castle = true,
                        'Q' => result.white.can_queen_castle = true,
                    
                        _ => return None,
                    }
                }
            } else if field.len() != 1 {
                return None;
            }
        }
    
    
        // Parse en passant availability
        if let Some(&field) = fields.get(3) {
            if Some('-') != field.chars().nth(0) {
                if field.len() != 2 {
                    return None;
                }
    
                let tile: Vec<char> = field.chars().collect();
                let file = match *tile.get(0)? {
                    'a' => 0,
                    'b' => 1,
                    'c' => 2,
                    'd' => 3,
                    'e' => 4,
                    'f' => 5,
                    'g' => 6,
                    'h' => 7,
                    _ => return None,
                };
                let rank = match *tile.get(1)? {
                    '3' => 5,
                    '6' => 2,
                    _ => return None,
                };
                result.board[rank * 8 + file].set_en_passant(true);
            } else if field.len() != 1 {
                return None;
            }
        }
    
    
        // Parse half move count
        if let Some(&field) = fields.get(4) {
            result.halfmove_count = field.parse().ok()?;
        }
        if let Some(&field) = fields.get(5) {
            result.fullmove_count = field.parse().ok()?;
            if result.fullmove_count == 0 {
                return None;
            }
        }
    
        Some(result)
    }
}
impl GameState {
    pub const FEN_CLASSIC: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
}
impl Default for GameState {
    fn default() -> Self {
        GameState {
            board: [Tile::empty(); 64],
            current: Color::White,
            black: PlayerState::default(),
            white: PlayerState::default(),
            halfmove_count: 0,
            fullmove_count: 0,
            en_passant: None,
        }
    }
}

