mod lib;
use lib::*;

mod known;

use std::path::Path;

use nanorand::{Rng, WyRand};


fn main() -> Result<(), std::io::Error> {
    let reference_path = Path::new(REFERENCE_PATH);
    let reference = if !reference_path.exists() || std::env::args().len() > 1 {
        println!("No reference found, creating new");
        let reference = create_initial();
        reference.write(reference_path)?;
        reference
    } else {
        println!("Using reference as base");
        ReferenceDatabase::read(reference_path)?
    };

    let database_path = Path::new(DATABASE_PATH);
    if cfg!(not(target_feature = "pext")) {
        let old_magics = if let Ok(old) = Database::read(database_path) {
            old.magics
        } else {
            reference.magics.clone()
        };

        //let magics = upgrade_magics(old_magics, &reference);
        let magics = old_magics;
        let database = build_database(magics, &reference);
        database.write(database_path)?;

        for (index, magic) in database.magics.iter().skip(64).enumerate() {
            for _ in 0..5 {//let b = WyRand::new().generate();
                let occ: u64 = nanorand::WyRand::new().generate();
                let occ = magic.mask;
                debug_print(index, database.sliding_table[magic.key(occ)], occ, 1000);
            }
        }
    } else {
        let database = build_database(reference.magics.clone(), &reference);
        database.write(database_path)?;
    }

    //database.write(database_path)?;
    Ok(())
}



fn create_initial() -> ReferenceDatabase {
    let mut moves = Vec::new();
    let mut occluders = Vec::new();
    let mut magics: Vec<MagicSquare> = Vec::new();

    let mut offset = 0;
    let mut create = |
        occlusion_gen: &dyn Fn(usize) -> u64, 
        attack_gen: &dyn Fn(usize, u64) -> u64, 
        known_values: Option<&([usize; 64], [u64; 64])>,
        debug: bool,
    | {
        for square in 0..64 {
            // Create the magic square with relevant info
            // the magic number will be found later
            let mask = occlusion_gen(square);
            let (shift, value) = known_values
                .and_then(|(shifts, values)| {
                    Some((64 - shifts[square], values[square]))
                })
                .unwrap_or((64 - mask.count_ones() as usize, 0));
            magics.push(MagicSquare {
                offset,
                mask,
                shift,
                value,
            });

            // Iterate through all possible combinations of blockers, 
            // that are a subset of the occlusion mask
            let mut occlusion = 0;
            loop {
                let attacks = attack_gen(square, occlusion);
                moves.push(attacks);
                occluders.push(occlusion);

                offset += 1;
                occlusion = occlusion.wrapping_sub(mask) & mask;
                /*if occlusion == mask && debug {
                    debug_print(square, attacks, mask, 1000);
                }*/
                if occlusion == 0 {
                    break;
                }
            }
            if debug {
                //debug_print(square, moves[magics.last().unwrap().key(mask)], 0, 1000);
            }
        }
    };

    create(
        &generate_rook_occlusion_mask, 
        &generate_rook_attack_mask,
        Some(&known::KNOWN_ROOK_MAGIC_VALUES),
        false,
    );
    create(
        &generate_bishop_occlusion_mask, 
        &generate_bishop_attack_mask,
        Some(&known::KNOWN_BISHOP_MAGIC_VALUES),
        true,
    );

    ReferenceDatabase {
        magics,
        moves,
        occluders,
    }
}


///
/// Creates a move database using a reference database
/// 
fn build_database(magics: Vec<MagicSquare>, reference: &ReferenceDatabase) -> Database {
    let mut moves = Vec::new();

    let offsets: Vec<usize> = reference.magics.iter().map(|magic| magic.offset).collect();

    let mut sizes: Vec<usize> = reference.magics.iter().skip(1).map(|magic| magic.offset).collect();


    sizes.push(reference.moves.len());

    let iter = magics.iter().zip(offsets.iter().zip(sizes.iter()));
    for (magic, (&offset, &size)) in iter {
        let moves_ref = reference.moves.iter()
            .skip(offset)
            .take(size - offset);
        
        let occluders_ref = reference.occluders.iter()
            .skip(offset)
            .take(size - offset);
        
        for (&movement, &occlusion) in moves_ref.zip(occluders_ref) {
            let key = magic.key(occlusion);
            if key >= moves.len() {
                moves.resize(key + 1, 0);
            }
            moves[key] = movement;
        }
    }

    Database {
        magics,
        sliding_table: moves,

        pawns: (0..64).map(generate_pawn_attack_mask).flatten().collect(),
        knights: (0..64).map(generate_knight_attack_mask).collect(),
        kings: (0..64).map(generate_king_attack_mask).collect()
    }
}


// https://github.com/maksimKorzh/chess_programming/blob/master/src/bbc/init_magics/bbc.c
fn upgrade_magics(mut magics: Vec<MagicSquare>, reference: &ReferenceDatabase) -> Vec<MagicSquare> {
    for magic in &mut magics {
        magic.shift = if magic.value == 0 {
            magic.shift
        } else {
            (magic.shift + 1).max(64)
        };


        let mut rng = WyRand::new_seed(0);
        'outer: loop {
            // Generate a random magic with a high-ish population count
            loop {
                magic.value = rng.generate::<u64>();
                if ((magic.value * magic.value) >> 56).count_ones() > 5 {
                    break;
                }
            }

            let mut used_moves = vec![0; 0x1000];

            // Test the random magic to see ...
            let iter = reference.moves.iter().skip(magic.offset).zip(
                reference.occluders.iter().skip(magic.offset)
            );
            for (i, (&truth, &occlusion)) in iter.enumerate() {
                if i >= (0x1000 - 1) {
                    break 'outer;
                }
                let key = magic.key(occlusion) as usize;
                let index = key - magic.offset;
                
                if used_moves[index] == 0 {
                    // Magic has not yet been used
                    used_moves[index] = truth;
                } else if used_moves[index] != truth {
                    // Magic has been used, but with a bad collision
                    continue 'outer;
                }
            }
            break;
        }
        println!("magic found: {:x}", magic.value);
    }

    magics
}





fn generate_rook_occlusion_mask(square: usize) -> u64 {
    let occupancy = 1 << square;
    (
        generate_line_mask(occupancy, 0, Direction::North) & NOT_RANK_1 | 
        generate_line_mask(occupancy, 0, Direction::South) & NOT_RANK_8 | 
        generate_line_mask(occupancy, 0, Direction::East) & NOT_FILE_H | 
        generate_line_mask(occupancy, 0, Direction::West) & NOT_FILE_A
    ) & !occupancy
}
fn generate_rook_attack_mask(square: usize, occlusion: u64) -> u64 {
    let occupancy = 1 << square;
    (
        generate_line_mask(occupancy, occlusion, Direction::North) | 
        generate_line_mask(occupancy, occlusion, Direction::South) | 
        generate_line_mask(occupancy, occlusion, Direction::East) | 
        generate_line_mask(occupancy, occlusion, Direction::West)
    ) & !occupancy
}

fn generate_bishop_occlusion_mask(square: usize) -> u64 {
    let occupancy = 1 << square;
    (
        generate_line_mask(occupancy, 0, Direction::NorthEast) & (NOT_RANK_1 & NOT_FILE_H) | 
        generate_line_mask(occupancy, 0, Direction::NorthWest) & (NOT_RANK_1 & NOT_FILE_A) | 
        generate_line_mask(occupancy, 0, Direction::SouthEast) & (NOT_RANK_8 & NOT_FILE_H) | 
        generate_line_mask(occupancy, 0, Direction::SouthWest) & (NOT_RANK_8 & NOT_FILE_A) 
    ) & !occupancy
}
fn generate_bishop_attack_mask(square: usize, occlusion: u64) -> u64 {
    let occupancy = 1 << square;
    (
        generate_line_mask(occupancy, occlusion, Direction::NorthEast) | 
        generate_line_mask(occupancy, occlusion, Direction::NorthWest) | 
        generate_line_mask(occupancy, occlusion, Direction::SouthEast) | 
        generate_line_mask(occupancy, occlusion, Direction::SouthWest)
    ) & !occupancy
}

fn generate_pawn_attack_mask(square: usize) -> [u64; 2] {
    let occupancy = 1 << square;
    let white = (occupancy >> 7) & NOT_FILE_A | (occupancy >> 9) & NOT_FILE_H;
    let black = (occupancy << 9) & NOT_FILE_A | (occupancy << 7) & NOT_FILE_H;
    [white, black]
}
fn generate_knight_attack_mask(square: usize) -> u64 {
    let occupancy = 1 << square;
    let north_east: u64 = (occupancy << 1) & NOT_FILE_A;
    let north_west: u64 = (occupancy >> 1) & NOT_FILE_H;
    let south_east: u64 = (occupancy << 2) & NOT_FILE_AB;
    let south_west: u64 = (occupancy >> 2) & NOT_FILE_GH;
    let north: u64 = north_east | north_west;
    let south: u64 = south_east | south_west;
    
    {
        (north << 16) | 
        (north >> 16) | 
        (south << 8) | 
        (south >> 8)
    }
}
fn generate_king_attack_mask(square: usize) -> u64 {
    let occupancy = 1 << square;
    {
        // north and south
        occupancy >> 8 |
        occupancy << 8 |
        // east and west
        (occupancy << 1) & NOT_FILE_A |
        (occupancy >> 1) & NOT_FILE_H | 
        // north east and west
        (occupancy >> 7) & NOT_FILE_A |
        (occupancy >> 9) & NOT_FILE_H |
        // south east and west
        (occupancy << 9) & NOT_FILE_A |
        (occupancy << 7) & NOT_FILE_H
    }
}




#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Direction {
    North,
    South,
    East, 
    West, 
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

const NOT_FILE_A: u64 = 0xfefefefefefefefe;
const NOT_FILE_H: u64 = 0x7f7f7f7f7f7f7f7f;
const NOT_FILE_AB: u64 = 0xfcfcfcfcfcfcfcfc;
const NOT_FILE_GH: u64 = 0x3f3f3f3f3f3f3f3f;

const NOT_RANK_1: u64 = 0xffffffffffffff00;
const NOT_RANK_8: u64 = 0x00ffffffffffffff;


fn generate_line_mask(mut occupancy: u64, mut occlusion: u64, direction: Direction) -> u64 {
    let (left_shift, right_shift, mask) = match direction {
        Direction::North        => (0, 8, !0),
        Direction::South        => (8, 0, !0),
        Direction::East         => (1, 0, NOT_FILE_A),
        Direction::West         => (0, 1, NOT_FILE_H),
        Direction::NorthEast    => (0, 7, NOT_FILE_A),
        Direction::NorthWest    => (0, 9, NOT_FILE_H),
        Direction::SouthEast    => (9, 0, NOT_FILE_A),
        Direction::SouthWest    => (7, 0, NOT_FILE_H),
    };
    let shift = |value: u64, multiplier: usize| -> u64 {
        (value >> (right_shift * multiplier)) << (left_shift * multiplier)
    };

    occlusion = mask & !shift(occlusion & !occupancy, 1);
    occupancy |= occlusion & shift(occupancy, 1);
    occlusion &= shift(occlusion, 1);
    occupancy |= occlusion & shift(occupancy, 2);
    occlusion &= shift(occlusion, 2);
    occupancy |= occlusion & shift(occupancy, 4);
    occupancy
}




fn debug_print(square: usize, movement: u64, occlusion: u64, timeout: u64) {
    print!("{esc}[2J{esc}[0;0H", esc = 27 as char);
    for i in 0..64 {
        if i % 8 == 0 && i != 0 {
            println!("");
        }

        let is_occluded = occlusion & (1 << i) != 0;
        let is_attacking = movement & (1 << i) != 0;

        let sym = if i == square {
            " * "
        } else if is_attacking && is_occluded {
            " X "
        } else if is_occluded {
            " # "
        } else if is_attacking {
            " - "
        } else {
            " . "
        };

        print!("{}", sym);
    }
    println!("");
    std::thread::sleep(std::time::Duration::from_millis(timeout))
}