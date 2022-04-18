use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

use nanoserde::*;



#[derive(Copy, Clone, Default, Debug, SerBin, DeBin)]
pub struct MagicSquare {
    // The offset into the sliding table, where this square starts
    pub offset: usize,

    // The bitshift attributed to this square
    pub shift: usize,
    
    // The attacking mask for this square
    pub mask: u64,
    
    // The magic value used in combination with the bitshift to generate the index
    pub value: u64,
}
impl MagicSquare {
    pub fn key(&self, occlusion: u64) -> usize {
        let key = if cfg!(target_feature = "pext") {
            0_u64
            //self.offset + board.pext(self.mask) as usize
        } else {
            self.offset as u64 + (((occlusion & self.mask).wrapping_mul(self.value)) >> self.shift as u64 )
        };
        key as usize
    }
}


pub trait Cacheable where Self: Sized + SerBin + DeBin {
    fn read(path: &Path) -> Result<Self, std::io::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let result = Self::deserialize_bin(&buffer[..]).or_else(|error| {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("Failed to deserialize database:\n{}", error)
            ))
        });
        Ok(result?)
    }

    fn write(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let mut buffer = self.serialize_bin();
        file.write_all(&mut buffer)?;
        Ok(())
    }
}


#[derive(Clone, Default, Debug, SerBin, DeBin)]
pub struct ReferenceDatabase {
    pub magics: Vec<MagicSquare>,
    pub moves: Vec<u64>,
    pub occluders: Vec<u64>,
}
impl Cacheable for ReferenceDatabase {}


#[derive(Clone, Default, Debug, SerBin, DeBin)]
pub struct Database {
    // 0-63     rook squares
    // 64-127   bishop squares
    pub magics: Vec<MagicSquare>,

    //database: Vec<(u64, Moves)>,
    pub sliding_table: Vec<u64>,

    pub pawns: Vec<u64>,
    pub knights: Vec<u64>,
    pub kings: Vec<u64>,
}
impl Cacheable for Database {}



pub const DATABASE_PATH: &'static str = "./wizard/database.bin";
pub const REFERENCE_PATH: &'static str = "./wizard/reference.bin";


pub fn read_database(path: Option<&Path>) -> Result<Database, std::io::Error> {
    Database::read(path.unwrap_or(Path::new(DATABASE_PATH)))
}
