use crate::*;

use nanorand::Rng;
use bitintr::Pext;
use pretty_assertions::{
    assert_eq,
    assert_ne,
};
//macro_rules! assert_eq { ($left:expr, $right:expr$(,)?) => { $left }; }



#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct MoveCountInfo {
    nodes: usize,
    captures: usize,
    en_passants: usize,
    castles: usize,
    promotions: usize,
    checks: usize,
    discovery_checks: usize,
    double_checks: usize,
    checkmates: usize,
}
impl std::ops::Add for MoveCountInfo {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            nodes: self.nodes + rhs.nodes,
            captures: self.captures + rhs.captures,
            en_passants: self.en_passants + rhs.en_passants,
            castles: self.castles + rhs.castles,
            promotions: self.promotions + rhs.promotions,
            checks: self.checks + rhs.checks,
            discovery_checks: self.discovery_checks + rhs.discovery_checks,
            double_checks: self.double_checks + rhs.double_checks,
            checkmates: self.checkmates + rhs.checkmates,
        }
    }
}


// https://www.chessprogramming.org/Perft_Results
fn get_move_count(state: GameState, depth: usize) -> MoveCountInfo {
    /*for (index, tile) in state.board.iter().enumerate() {
        if index % 8 == 0 {
            println!("");
        }
        print!(" {} ", if tile.is_occupied() { "#" } else {"."})
    }
    println!("");*/

    let mut info = MoveCountInfo::default();

    let pieces = match state.current {
        Color::White => &state.white.pieces,
        Color::Black => &state.black.pieces,
    };

    if depth > 0 {
        for (_, _, moves) in pieces {
            for movement in moves {
                if movement.is_capture() {
                    /*if let (PieceType::King, _) = state.board[movement.get_target() as usize].get_piece() {
                        panic!("King capture\n{}", state.to_fen());
                    }*/
                    if state.current == state.board[movement.get_target() as usize].get_color() {
                        panic!("Self capture\n{}", state.to_fen());
                    }
                    info.captures += 1;
                }
                if movement.is_promotion() {
                    info.promotions += 1;
                }

                let mut state = state.clone();
                state.play_move_unchecked(*movement);
                state.current = !state.current;
                state.generate_moves(state.current);
                //state.validate_moves();
                
                if state.is_king_checked(state.current) {
                    info.checks += 1;
                    if state.is_king_checkmated(state.current) {
                        info.checkmates += 1;
                    }
                }

                if !state.is_king_checked(!state.current) {
                    info = info + get_move_count(state, depth - 1);
                }
            }
        }
        info
    } else {
        MoveCountInfo {
            nodes: 1,
            ..Default::default()
        }
    }
}




macro_rules! create_perft_testbench {
    ($ply:expr, $name:ident, $($info:tt)*) => {
        fn $name(criterion: &mut Criterion) {
            let mut state = GameState::from_fen(GameState::FEN_CLASSIC).expect("Fen parsing failed. Use fen tests");
            state.generate_moves(state.current);

            criterion.bench_function(concat!("perft for ply ", stringify!($ply)), |bencher| bencher.iter(| | {
                assert_eq!(get_move_count(state.clone(), $ply), $($info)*);
            }));
        }
    };
}




fn generate_pseudo_legal_moves(criterion: &mut Criterion) {
    let mut state = GameState::from_fen(GameState::FEN_CLASSIC).expect("Fen parsing failed. Use fen tests");

    criterion.bench_function("generate pseudo legal moves", |bencher| bencher.iter(| | {
        state.generate_moves(Color::White);
        state.generate_moves(Color::Black);
    }));
}
fn generate_legal_moves(criterion: &mut Criterion) {
    let mut state = GameState::from_fen(GameState::FEN_CLASSIC).expect("Fen parsing failed. Use fen tests");

    criterion.bench_function("generate legal moves", |bencher| bencher.iter(| | {
        state.generate_moves(Color::White);
        state.generate_moves(Color::Black);
        //state.validate_moves();
    }));
}

fn move_validation(criterion: &mut Criterion) {
    let mut state = GameState::from_fen(GameState::FEN_CLASSIC).expect("Fen parsing failed. Use fen tests");
    state.generate_moves(Color::White);
    state.generate_moves(Color::Black);

    criterion.bench_function("move validation", |bencher| bencher.iter(| | {
        let mut state = state.clone();
        //state.validate_moves();
    }));
}

fn generate_king_check_mask(criterion: &mut Criterion) {
    let mut state = GameState::from_fen(GameState::FEN_CLASSIC).expect("Fen parsing failed. Use fen tests");
    state.generate_moves(Color::White);
    state.generate_moves(Color::Black);
    let state = state.clone();

    criterion.bench_function("generate_king_check_mask", |bencher| bencher.iter(| | {
        state.generate_king_masks(Color::White);
        state.generate_king_masks(Color::Black);
    }));
}


create_perft_testbench!(0, perft_ply_0,
    MoveCountInfo {
        nodes: 1,
        captures: 0,
        en_passants: 0,
        castles: 0,
        promotions: 0,
        checks: 0,
        discovery_checks: 0,
        double_checks: 0,
        checkmates: 0,
    }
);

create_perft_testbench!(1, perft_ply_1,
    MoveCountInfo {
        nodes: 20,
        captures: 0,
        en_passants: 0,
        castles: 0,
        promotions: 0,
        checks: 0,
        discovery_checks: 0,
        double_checks: 0,
        checkmates: 0,
    }
);


create_perft_testbench!(2, perft_ply_2,
    MoveCountInfo {
        nodes: 400,
        captures: 0,
        en_passants: 0,
        castles: 0,
        promotions: 0,
        checks: 0,
        discovery_checks: 0,
        double_checks: 0,
        checkmates: 0,
    }
);

create_perft_testbench!(3, perft_ply_3,
    MoveCountInfo {
        nodes: 8902,
        captures: 34,
        en_passants: 0,
        castles: 0,
        promotions: 0,
        checks: 12,
        discovery_checks: 0,
        double_checks: 0,
        checkmates: 0,
    }
);

create_perft_testbench!(4, perft_ply_4,
    MoveCountInfo {
        nodes: 197281,
        captures: 1576,
        en_passants: 0,
        castles: 0,
        promotions: 0,
        checks: 469,
        discovery_checks: 0,
        double_checks: 0,
        checkmates: 8,
    }
);

create_perft_testbench!(5, perft_ply_5,
    MoveCountInfo {
        nodes: 197281,
        captures: 1576,
        en_passants: 0,
        castles: 0,
        promotions: 0,
        checks: 469,
        discovery_checks: 0,
        double_checks: 0,
        checkmates: 8,
    }
);


fn pext(criterion: &mut Criterion) {
    let mut rng = nanorand::WyRand::new();
    let pos = bitboard::Bitboard(1 << 28);
    let blockers: u64 = rng.generate();
    let tile = rng.generate_range(0..64);

    let mut path = std::path::PathBuf::new();
    path.push("../");
    path.push(wizard::DATABASE_PATH);
    let database = wizard::read_database(Some(path.as_path())).unwrap();

    criterion.bench_function("pext", |bencher| bencher.iter(| | {
        unsafe {
            let aaaaa = database.sliding_table[database.magics[tile].key(blockers)];
        }
    }));
}
fn hashmap(criterion: &mut Criterion) {
    let mut map = std::collections::HashMap::new();

    let mut squares = Vec::new();

    for square in 0..64 {
        let mask = bitboard::PlayerBitboard::generate_rook_attacks(bitboard::Bitboard(1 << square as u64), bitboard::Bitboard(0)).0
            & !bitboard::Bitboard::RANK_MASK[0]
            & !bitboard::Bitboard::RANK_MASK[7]
            & !bitboard::Bitboard::FILE_MASK[0]
            & !bitboard::Bitboard::FILE_MASK[7]
            & !(1 << square as u64);

        squares.push(mask);

        let mut blockers = 0;
        loop {
            let attacks = bitboard::PlayerBitboard::generate_rook_attacks(bitboard::Bitboard(1 << square as u64), bitboard::Bitboard(blockers)).0;
            map.insert(blockers, attacks);
            
            blockers = (blockers - mask) & mask;
            if blockers == 0 {
                break;
            }
        }
    }


    let blockers = bitboard::Bitboard(nanorand::WyRand::new().generate()).0;
    criterion.bench_function("hashmap", |bencher| bencher.iter(| | {
        map.get(&(blockers & squares[28])).unwrap();
    }));
}
fn board(criterion: &mut Criterion) {
    let pos = bitboard::Bitboard(1 << 28);
    let blockers = bitboard::Bitboard(!0);//bitboard::Bitboard(nanorand::WyRand::new().generate());

    criterion.bench_function("board", |bencher| bencher.iter(| | {
        bitboard::PlayerBitboard::generate_rook_attacks(pos, blockers);
    }));
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .plotting_backend(criterion::PlottingBackend::Plotters)
        //.sample_size(10000)
        //.measurement_time(std::time::Duration::from_secs(5))
        ;
    targets = 
        //generate_pseudo_legal_moves,
        //generate_king_check_mask,
        //generate_legal_moves,
        pext,
        board,
        hashmap,
        //move_validation,
        //perft_ply_0,
        //perft_ply_1,
        //perft_ply_2,
        //perft_ply_3,
        //perft_ply_4,
        //perft_ply_5,
);
