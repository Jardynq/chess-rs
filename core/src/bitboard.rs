use crate::*;



#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
// TODO: pub type Bitboard = u64;
pub struct Bitboard(pub u64);
impl Bitboard {
    pub const FILE_MASK: [u64; 8] = [
        0x0101010101010101,
        0x0202020202020202,
        0x0404040404040404,
        0x0808080808080808,
        0x1010101010101010,
        0x2020202020202020,
        0x4040404040404040,
        0x8080808080808080,
    ];

    // TODO currently 0,0 is top left bit it should be bottom left!!!!!
    pub const RANK_MASK: [u64; 8] = [
        0x00000000000000FF,
        0x000000000000FF00,
        0x0000000000FF0000,
        0x00000000FF000000,
        0x000000FF00000000,
        0x0000FF0000000000,
        0x00FF000000000000,
        0xFF00000000000000,
    ];
}
impl Bitboard {
    pub fn mirror_vertical(self) -> Self {
        const K1: u64 = 0x00FF00FF00FF00FF;
        const K2: u64 = 0x0000FFFF0000FFFF;

        let mut result: u64 = self.0;
        result = ((result >>  8) & K1) | ((result & K1) <<  8);
        result = ((result >> 16) & K2) | ((result & K2) << 16);
        result = ( result >> 32)       | ( result       << 32);
        return Self(result);
    }

    pub fn set_bit(&mut self, coord: Coord) {
        if let Some(index) = coord.index() {
            self.0 |= 1 << index;
        }
    }
    pub fn unset_bit(&mut self, coord: Coord) {
        if let Some(index) = coord.index() {
            self.0 &= !(1 << index);
        }
    }
    pub fn is_occupied(&self, index: usize) -> bool {
        (1 << index) & self.0 != 0
    }



    // TODO: removing self form occlusion is a bad idea for more than one piece, since ut then can attack trhough other pieces of same type
    // But at the same time, this does not matter since, the occluding piece would just continue the ray anyways.
    pub fn fill_north_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = !((occlusion.0 & !self.0) >> 8);
        self.0 |= occlusion.0 & (self.0 >> 8);
        occlusion.0 &= occlusion.0 >> 8;
        self.0 |= occlusion.0 & (self.0 >> 16);
        occlusion.0 &= occlusion.0 >> 16;
        self.0 |= occlusion.0 & (self.0 >> 32);
        self
    }
    pub fn fill_south_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = !((occlusion.0 & !self.0) << 8);
        self.0 |= occlusion.0 & (self.0 << 8);
        occlusion.0 &= occlusion.0 << 8;
        self.0 |= occlusion.0 & (self.0 << 16);
        occlusion.0 &= occlusion.0 << 16;
        self.0 |= occlusion.0 & (self.0 << 32);
        self
    }
    pub fn fill_east_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = {
            !((occlusion.0 & !self.0) << 1) &
            !Bitboard::FILE_MASK[0] // Mask to avoid wrapping
        };
        self.0 |= occlusion.0 & (self.0 << 1);
        occlusion.0 &= occlusion.0 << 1;
        self.0 |= occlusion.0 & (self.0 << 2);
        occlusion.0 &= occlusion.0 << 2;
        self.0 |= occlusion.0 & (self.0 << 4);
        self
    }
    pub fn fill_west_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = {
            !((occlusion.0 & !self.0) >> 1) &
            !Bitboard::FILE_MASK[7] // Mask to avoid wrapping
        };
        self.0 |= occlusion.0 & (self.0 >> 1);
        occlusion.0 &= occlusion.0 >> 1;
        self.0 |= occlusion.0 & (self.0 >> 2);
        occlusion.0 &= occlusion.0 >> 2;
        self.0 |= occlusion.0 & (self.0 >> 4);
        self
    }
    pub fn fill_north_east_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = {
            !((occlusion.0 & !self.0) >> 7) &
            !Bitboard::FILE_MASK[0] // Mask to avoid wrapping
        };
        self.0 |= occlusion.0 & (self.0 >> 7);
        occlusion.0 &= occlusion.0 >> 7;
        self.0 |= occlusion.0 & (self.0 >> 14);
        occlusion.0 &= occlusion.0 >> 14;
        self.0 |= occlusion.0 & (self.0 >> 28);
        self
    }
    pub fn fill_north_west_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = {
            !((occlusion.0 & !self.0) >> 9) &
            !Bitboard::FILE_MASK[7] // Mask to avoid wrapping
        };
        self.0 |= occlusion.0 & (self.0 >> 9);
        occlusion.0 &= occlusion.0 >> 9;
        self.0 |= occlusion.0 & (self.0 >> 18);
        occlusion.0 &= occlusion.0 >> 18;
        self.0 |= occlusion.0 & (self.0 >> 36);
        self
    }
    pub fn fill_south_east_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = {
            !((occlusion.0 & !self.0) << 9) &
            !Bitboard::FILE_MASK[0] // Mask to avoid wrapping
        };
        self.0 |= occlusion.0 & (self.0 << 9);
        occlusion.0 &= occlusion.0 << 9;
        self.0 |= occlusion.0 & (self.0 << 18);
        occlusion.0 &= occlusion.0 << 18;
        self.0 |= occlusion.0 & (self.0 << 36);
        self
    }
    pub fn fill_south_west_occluded(mut self, mut occlusion: Self) -> Self {
        occlusion.0 = {
            !((occlusion.0 & !self.0) << 7) &
            !Bitboard::FILE_MASK[7] // Mask to avoid wrapping
        };
        self.0 |= occlusion.0 & (self.0 << 7);
        occlusion.0 &= occlusion.0 << 7;
        self.0 |= occlusion.0 & (self.0 << 14);
        occlusion.0 &= occlusion.0 << 14;
        self.0 |= occlusion.0 & (self.0 << 28);
        self
    }

    
    pub fn fill_north(mut self) -> Self {
        self.0 |= self.0 >>  8;
        self.0 |= self.0 >> 16;
        self.0 |= self.0 >> 32;
        return self;
    }
    pub fn fill_south(mut self) -> Self {
        self.0 |= self.0 <<  8;
        self.0 |= self.0 << 16;
        self.0 |= self.0 << 32;
        return self;
    }
    pub fn fill_east(mut self) -> Self {
        // Masks to avoid wrapping
        const MASK_A: u64 = !Bitboard::FILE_MASK[0];
        const MASK_B: u64 = MASK_A & (MASK_A << 1);
        const MASK_C: u64 = MASK_B & (MASK_B << 2);

        self.0 |= MASK_A & (self.0 << 1);
        self.0 |= MASK_B & (self.0 << 2);
        self.0 |= MASK_C & (self.0 << 4);
        self
    }
    pub fn fill_west(mut self) -> Self {
        // Masks to avoid wrapping
        const MASK_A: u64 = !Bitboard::FILE_MASK[7];
        const MASK_B: u64 = MASK_A & (MASK_A >> 1);
        const MASK_C: u64 = MASK_B & (MASK_B >> 2);

        self.0 |= MASK_A & (self.0 >> 1);
        self.0 |= MASK_B & (self.0 >> 2);
        self.0 |= MASK_C & (self.0 >> 4);
        self
    }

    pub fn fill_north_east(mut self) -> Self {
        // Masks to avoid wrapping
        const MASK_A: u64 = !Bitboard::FILE_MASK[0];
        const MASK_B: u64 = MASK_A & (MASK_A >>  7);
        const MASK_C: u64 = MASK_B & (MASK_B >> 14);

        self.0 |= MASK_A & (self.0 >>  7);
        self.0 |= MASK_B & (self.0 >> 14);
        self.0 |= MASK_C & (self.0 >> 28);
        self
    }
    pub fn fill_north_west(mut self) -> Self {
        // Masks to avoid wrapping
        const MASK_A: u64 = !Bitboard::FILE_MASK[7];
        const MASK_B: u64 = MASK_A & (MASK_A >>  9);
        const MASK_C: u64 = MASK_B & (MASK_B >> 18);

        self.0 |= MASK_A & (self.0 >>  9);
        self.0 |= MASK_B & (self.0 >> 18);
        self.0 |= MASK_C & (self.0 >> 36);
        self
    }
    pub fn fill_south_east(mut self) -> Self {
        // Masks to avoid wrapping
        const MASK_A: u64 = !Bitboard::FILE_MASK[7];
        const MASK_B: u64 = MASK_A & (MASK_A <<  7);
        const MASK_C: u64 = MASK_B & (MASK_B << 14);

        self.0 |= MASK_A & (self.0 <<  7);
        self.0 |= MASK_B & (self.0 << 14);
        self.0 |= MASK_C & (self.0 << 28);
        self
    }
    pub fn fill_south_west(mut self) -> Self {
        // Masks to avoid wrapping
        const MASK_A: u64 = !Bitboard::FILE_MASK[0];
        const MASK_B: u64 = MASK_A & (MASK_A <<  9);
        const MASK_C: u64 = MASK_B & (MASK_B << 18);

        self.0 |= MASK_A & (self.0 <<  9);
        self.0 |= MASK_B & (self.0 << 18);
        self.0 |= MASK_C & (self.0 << 36);
        self
    }


    pub fn count() {

    }
}
impl std::ops::Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
macro_rules! impl_bitwise {
    ($impl_for:ty: $operator:tt $($to_impl:path)*, $($function:tt)* ) => {
        impl $($to_impl)* for $impl_for {
            type Output = Self;
            fn $($function)*(self, rhs: Self) -> Self::Output {
                Self(self.0 $operator rhs.0)
            }
        }
    };
}

impl_bitwise!(Bitboard: + std::ops::Add, add);
impl_bitwise!(Bitboard: - std::ops::Sub, sub);


impl std::ops::BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl std::ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl std::ops::BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl std::ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl std::ops::BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}
impl std::ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}





#[derive(Clone, Copy, Default, Debug)]
pub struct PlayerBitboard {
    pub pawns: Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks: Bitboard,
    pub queens: Bitboard,
    pub king: Bitboard,
}
impl PlayerBitboard {
    pub fn occupancy(&self) -> Bitboard {
        self.pawns |
        self.knights |
        self.bishops |
        self.rooks |
        self.queens
        //self.king
    }
    pub fn attacks(&self, mut occlusion: Bitboard, color: Color) -> Bitboard {
        occlusion |= self.occupancy();
        Self::generate_pawn_attacks(self.pawns, color) |
        Self::generate_knight_attacks(self.knights) |
        Self::generate_bishop_attacks(self.bishops, occlusion) |
        Self::generate_rook_attacks(self.rooks, occlusion) |
        Self::generate_queen_attacks(self.queens, occlusion) |
        Self::generate_king_attacks(self.king)
    }

    pub fn set_bit(&mut self, piece_type: PieceType, coord: Coord) {
        match piece_type {
            PieceType::Pawn => self.pawns.set_bit(coord),
            PieceType::Knight => self.knights.set_bit(coord),
            PieceType::Bishop => self.bishops.set_bit(coord),
            PieceType::Rook => self.rooks.set_bit(coord),
            PieceType::Queen => self.queens.set_bit(coord),
            PieceType::King => self.king.set_bit(coord),
            PieceType::None => (),
        }
    }
    pub fn unset_bit(&mut self, piece_type: PieceType, coord: Coord) {
        match piece_type {
            PieceType::Pawn => self.pawns.unset_bit(coord),
            PieceType::Knight => self.knights.unset_bit(coord),
            PieceType::Bishop => self.bishops.unset_bit(coord),
            PieceType::Rook => self.rooks.unset_bit(coord),
            PieceType::Queen => self.queens.unset_bit(coord),
            PieceType::King => self.king.unset_bit(coord),
            PieceType::None => (),
        }
    }



    pub fn generate_pawn_attacks(pieces: Bitboard, color: Color) -> Bitboard {
        match color {
            Color::White => {
                // north east and west
                Bitboard(
                    (pieces.0 >> 7) & !Bitboard::FILE_MASK[0] |
                    (pieces.0 >> 9) & !Bitboard::FILE_MASK[7]
                )
            }
            Color::Black => {
                // south east and west
                Bitboard(
                    (pieces.0 << 9) & !Bitboard::FILE_MASK[0] |
                    (pieces.0 << 7) & !Bitboard::FILE_MASK[7]
                )
            }
        }
    }
    pub fn generate_pawn_movement(pieces: Bitboard, color: Color, mut occlusion: Bitboard) -> Bitboard {
        match color {
            Color::White => {
                // north
                occlusion.0 = occlusion.0 | occlusion.0 >> 8;
                Bitboard(
                    (
                        pieces.0 >> 8 |
                        ((Bitboard::RANK_MASK[6] & pieces.0) >> 16)
                    ) & !occlusion.0
                )
            }
            Color::Black => {
                // south
                occlusion.0 = occlusion.0 | occlusion.0 << 8;
                Bitboard(
                    (
                        pieces.0 << 8 |
                        ((Bitboard::RANK_MASK[6] & pieces.0) << 16)
                    ) & !occlusion.0
                )
            }
        }
    }
    pub fn generate_knight_attacks(pieces: Bitboard) -> Bitboard {
        let north_east: u64 = (pieces.0 << 1) & 0xfefefefefefefefe;
        let north_west: u64 = (pieces.0 >> 1) & 0x7f7f7f7f7f7f7f7f;
        let south_east: u64 = (pieces.0 << 2) & 0xfcfcfcfcfcfcfcfc;
        let south_west: u64 = (pieces.0 >> 2) & 0x3f3f3f3f3f3f3f3f;
        let north: u64 = north_east | north_west;
        let south: u64 = south_east | south_west;
        
        Bitboard(
            (north << 16) | 
            (north >> 16) | 
            (south << 8) | 
            (south >> 8)
        )
    }
    pub fn generate_bishop_attacks(pieces: Bitboard, occlusion: Bitboard) -> Bitboard {
        pieces.fill_north_east_occluded(occlusion) |
        pieces.fill_north_west_occluded(occlusion) |
        pieces.fill_south_east_occluded(occlusion) |
        pieces.fill_south_west_occluded(occlusion)
    }
    pub fn generate_rook_attacks(pieces: Bitboard, occlusion: Bitboard) -> Bitboard {
        pieces.fill_north_occluded(occlusion) |
        pieces.fill_south_occluded(occlusion) |
        pieces.fill_east_occluded(occlusion) |
        pieces.fill_west_occluded(occlusion)
    }
    pub fn generate_queen_attacks(pieces: Bitboard, occlusion: Bitboard) -> Bitboard {
        pieces.fill_north_occluded(occlusion) |
        pieces.fill_south_occluded(occlusion) |
        pieces.fill_east_occluded(occlusion) |
        pieces.fill_west_occluded(occlusion) |
        pieces.fill_north_east_occluded(occlusion) |
        pieces.fill_north_west_occluded(occlusion) |
        pieces.fill_south_east_occluded(occlusion) |
        pieces.fill_south_west_occluded(occlusion)
    }
    pub fn generate_king_attacks(pieces: Bitboard) -> Bitboard {
        Bitboard(
            // north and south
            pieces.0 >> 8 |
            pieces.0 << 8 |
            // east and west
            (pieces.0 << 1) & !Bitboard::FILE_MASK[0] |
            (pieces.0 >> 1) & !Bitboard::FILE_MASK[7] | 
            // north east and west
            (pieces.0 >> 7) & !Bitboard::FILE_MASK[0] |
            (pieces.0 >> 9) & !Bitboard::FILE_MASK[7] |
            // south east and west
            (pieces.0 << 9) & !Bitboard::FILE_MASK[0] |
            (pieces.0 << 7) & !Bitboard::FILE_MASK[7]
        )
    }
}
