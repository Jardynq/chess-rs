use crate::*;
use bitintr::Pext;

#[derive(Copy, Clone, Default, Debug)]
pub struct MagicSquare {
    pub offset: usize,
    pub shift: usize,
    pub mask: u64,
    pub magic: u64,
}
impl MagicSquare {
    pub fn key(&self, board: u64) -> usize {
        if cfg!(target_feature = "pext") {
            self.offset + board.pext(self.mask) as usize
        } else {
            self.offset + (((board & self.mask) * self.magic) >> self.shift) as usize
        }
    }
}


// TODO include move array in database, so nothing really has to be generated
lazy_static::lazy_static!{
    pub static ref DATABASE: Vec<u64> = generate_magics().0;
    pub static ref ROOK_MAGIC: [MagicSquare; 64] = generate_magics().1;
    pub static ref BISHOP_MAGIC: [MagicSquare; 64] = [Default::default(); 64];
}    






// https://github.com/maksimKorzh/chess_programming/blob/master/src/bbc/init_magics/bbc.c
#[cfg(not(target_feature = "pext"))]
pub fn generate_magics() -> (Vec<u64>, [MagicSquare; 64]){
    let mut database = vec![0; 0x100000];
    let mut magics = Vec::new();

    // a rook or bishop can at max have 12 tiles occupied
    const MAX_SIZE: usize = 2usize.pow(12);
    let mut reference = Vec::with_capacity(MAX_SIZE);
    let mut occupancy = Vec::with_capacity(MAX_SIZE);
    let mut used_reference = vec![0; MAX_SIZE];

    let mut offset = 0;
    let mut count = 0;
    for square in 0..64 {
        // TODO masking is a bit bad, since a rook that on the edge should not be masked out
        let mut mask = PlayerBitboard::generate_rook_attacks(Bitboard(1 << square as u64), Bitboard(0)).0
            & !(1 << square as u64);

        if square == 0 {
            mask &= !Bitboard::RANK_MASK[7];
            mask &= !Bitboard::FILE_MASK[7];
        }
        else if square == 7 {
            mask &= !Bitboard::RANK_MASK[7];
            mask &= !Bitboard::FILE_MASK[0];
        }
        else if square == 56 {
            mask &= !Bitboard::RANK_MASK[0];
            mask &= !Bitboard::FILE_MASK[7];
        }
        else if square == 63 {
            mask &= !Bitboard::RANK_MASK[0];
            mask &= !Bitboard::FILE_MASK[0];
        }
        else if square > 0 && square < 7 {
            mask &= !Bitboard::RANK_MASK[7];
            mask &= !Bitboard::FILE_MASK[0];
            mask &= !Bitboard::FILE_MASK[7];
        }
        else if square > 56 && square < 63 {
            mask &= !Bitboard::RANK_MASK[0];
            mask &= !Bitboard::FILE_MASK[0];
            mask &= !Bitboard::FILE_MASK[7];
        }
        else if square % 8 == 0 && square != 0 && square != 56 {
            mask &= !Bitboard::RANK_MASK[0];
            mask &= !Bitboard::RANK_MASK[7];
            mask &= !Bitboard::FILE_MASK[7];
        }
        else if square % 8 == 7 && square != 7 && square != 63 {
            mask &= !Bitboard::RANK_MASK[0];
            mask &= !Bitboard::RANK_MASK[7];
            mask &= !Bitboard::FILE_MASK[0];
        }
        else {
            mask &= !Bitboard::RANK_MASK[0];
            mask &= !Bitboard::RANK_MASK[7];
            mask &= !Bitboard::FILE_MASK[0];
            mask &= !Bitboard::FILE_MASK[7];
        }



        //let shift = 64 - mask.count_ones() as usize;
        let shift = 64 - rook_relevant_bits[square];
        if mask.count_ones() as usize != rook_relevant_bits[square] {
            panic!("FFFFUCKAWD: {}, {}, {}", mask.count_ones(), rook_relevant_bits[square], square);
        }

        let mut result = MagicSquare {
            offset,
            mask,
            shift,
            magic: rook_magic_numbers[square],
        };

        // Loop through all possible rook attacks blocked by blockers on current square.
        let mut blockers = 0;
        loop {
            let attacks = PlayerBitboard::generate_rook_attacks(Bitboard(1 << square as u64), Bitboard(blockers)).0;
            let attacks = attacks & !(1 << square);
            reference.push(attacks);
            occupancy.push(blockers);
            
            unsafe {
                database[result.key(blockers)] = attacks;
            }
            
            offset += 1;
            blockers = (blockers - mask) & mask;
            if blockers == 0 {
                break;
            }
        }

        magics.push(result);
        

        // THIS SHIT DOES NOT WORK!!!!!!!!!!
        continue;

        // Time to find a magic that works
        println!("staring one");
        let mut rng = nanorand::WyRand::new_seed(0);
        'outer: loop {
            count += 1;

            // Generate a random magic with a high-ish population count
            loop {
                result.magic = rng.generate::<u64>();
                if ((result.magic * result.magic) >> 56).count_ones() > 5 {
                    break;
                }
            }

            used_reference = vec![0; MAX_SIZE];

            // Test the random magic to see ...
            for (&truth, &blockers) in reference.iter().zip(occupancy.iter()) {
                let key = result.key(blockers) as usize;
                let index = key - result.offset;
                
                if used_reference[index] == 0 {
                    // magic has not yet been used
                    used_reference[index] = truth;
                } else if used_reference[index] != truth {
                    continue 'outer;
                }
            }
            break;
        }
        println!("found one");
    }

    (database, magics.try_into().unwrap())
}


const rook_relevant_bits: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    12, 11, 11, 11, 11, 11, 11, 12
];
const rook_magic_numbers: [u64; 64] = [
    0x8a80104000800020,
    0x140002000100040,
    0x2801880a0017001,
    0x100081001000420,
    0x200020010080420,
    0x3001c0002010008,
    0x8480008002000100,
    0x2080088004402900,
    0x800098204000,
    0x2024401000200040,
    0x100802000801000,
    0x120800800801000,
    0x208808088000400,
    0x2802200800400,
    0x2200800100020080,
    0x801000060821100,
    0x80044006422000,
    0x100808020004000,
    0x12108a0010204200,
    0x140848010000802,
    0x481828014002800,
    0x8094004002004100,
    0x4010040010010802,
    0x20008806104,
    0x100400080208000,
    0x2040002120081000,
    0x21200680100081,
    0x20100080080080,
    0x2000a00200410,
    0x20080800400,
    0x80088400100102,
    0x80004600042881,
    0x4040008040800020,
    0x440003000200801,
    0x4200011004500,
    0x188020010100100,
    0x14800401802800,
    0x2080040080800200,
    0x124080204001001,
    0x200046502000484,
    0x480400080088020,
    0x1000422010034000,
    0x30200100110040,
    0x100021010009,
    0x2002080100110004,
    0x202008004008002,
    0x20020004010100,
    0x2048440040820001,
    0x101002200408200,
    0x40802000401080,
    0x4008142004410100,
    0x2060820c0120200,
    0x1001004080100,
    0x20c020080040080,
    0x2935610830022400,
    0x44440041009200,
    0x280001040802101,
    0x2100190040002085,
    0x80c0084100102001,
    0x4024081001000421,
    0x20030a0244872,
    0x12001008414402,
    0x2006104900a0804,
    0x1004081002402
];