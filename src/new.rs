// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// by Greg Floyd

// Imports --------------------------------------------------------------------
use {
    hashbrown::HashMap,
    std::{
        fs::File,
        io::{BufRead, BufReader},
        slice::Iter,
        sync::Arc,
        thread,
    },
};

// Types ----------------------------------------------------------------------
type NucleotideCounts = HashMap<NucleotideHash, u32>;

struct Genome<'a> {
    nucleotide: NucleotideHash,
    nucleotide_len: usize,
    bytes: Iter<'a, u8>,
}

#[derive(Hash, Default, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct NucleotideHash {
    key: u64,
}

// Constants ------------------------------------------------------------------
const NUCLEOTIDE_STRS: [&str; 5] = [
    "GGTATTTTAATTTATAGT",
    "GGTATTTTAATT",
    "GGTATT",
    "GGTA",
    "GGT",
];
const FILE_NAME: &str = "250000_in";
const FILE_START: &str = ">THREE";
const FILE_BUFFER_SIZE: usize = 65536;
const FILE_LINE_SIZE: usize = 80;

// Public Functions -----------------------------------------------------------
pub fn run() {
    let genome_arc = Arc::new(read_genome_file());

    let worker_threads: Vec<_> = NUCLEOTIDE_STRS
        .into_iter()
        .map(|nucleotide| {
            let genome = genome_arc.clone();
            thread::spawn(move || (nucleotide, count_nucleotides(nucleotide.len(), &genome)))
        })
        .collect();

    print_percentages(1, &genome_arc);
    print_percentages(2, &genome_arc);
    for thread in worker_threads.into_iter().rev() {
        if let Ok((nucleotide, counts)) = thread.join() {
            print_counts(nucleotide, &counts);
        }
    }
}

// Private Methods ------------------------------------------------------------
impl NucleotideHash {
    fn push_byte(&mut self, byte: u8, nucleotide_len: usize) {
        self.key <<= 2;
        self.key |= ((byte >> 1) & 0b11) as u64;
        self.key &= (1u64 << (2 * nucleotide_len)) - 1;
    }

    fn to_str(self, nucleotide_len: usize) -> String {
        let mut nucleotide_str = String::default();
        let mut hash_key = self.key as u8;
        for _ in 0..nucleotide_len {
            let char = match hash_key & 0b11 {
                0 => b'A',
                1 => b'C',
                2 => b'T',
                3 => b'G',
                _ => unreachable!(),
            };
            nucleotide_str.push(char.into());
            hash_key >>= 2;
        }
        nucleotide_str.chars().rev().collect()
    }

    fn from_str(nucleotide_str: &str) -> Self {
        let mut nucleotide = NucleotideHash::default();
        for byte in nucleotide_str.as_bytes() {
            nucleotide.push_byte(*byte, nucleotide_str.len());
        }
        nucleotide
    }
}

// Private Traits -------------------------------------------------------------
impl<'a> Iterator for Genome<'a> {
    type Item = NucleotideHash;

    fn next(&mut self) -> Option<NucleotideHash> {
        self.bytes.next().map(|&byte| {
            self.nucleotide.push_byte(byte, self.nucleotide_len);
            self.nucleotide
        })
    }
}

// Private Functions ----------------------------------------------------------
fn new_genome(nucleotide_len: usize, bytes: &[u8]) -> Genome {
    let mut bytes = bytes.iter();
    let mut nucleotide = NucleotideHash::default();
    for byte in bytes.by_ref().take(nucleotide_len - 1) {
        nucleotide.push_byte(*byte, nucleotide_len);
    }
    #[rustfmt::skip] { Genome {nucleotide, nucleotide_len, bytes} }
}

fn read_genome_file() -> Vec<u8> {
    let mut reader = BufReader::new(File::open(FILE_NAME).unwrap());
    let mut genome_bytes = Vec::with_capacity(FILE_BUFFER_SIZE);
    let mut line = String::with_capacity(FILE_LINE_SIZE);
    while let Ok(_) = reader.read_line(&mut line) {
        if line.starts_with(FILE_START) {
            break;
        }
        line.clear();
    }
    line.clear();
    while let Ok(b) = reader.read_line(&mut line) {
        if b == 0 {
            break;
        }
        let bytes = line.as_bytes();
        genome_bytes.extend(bytes[..bytes.len() - 1].iter());
        line.clear();
    }
    genome_bytes
}

fn count_nucleotides(nucleotide_len: usize, genome: &[u8]) -> NucleotideCounts {
    let mut count_table = NucleotideCounts::default();
    for nucleotide in new_genome(nucleotide_len, genome) {
        *count_table.entry(nucleotide).or_insert(0) += 1;
    }
    count_table
}

fn print_percentages(nucleotide_len: usize, genome: &[u8]) {
    let table = &count_nucleotides(nucleotide_len, &genome);
    let total_nucleotides = table.values().sum::<u32>() as f32;
    let mut sortable_table: Vec<_> = table.iter().collect();
    sortable_table.sort_by(|(_, count_lhs), (_, count_rhs)| count_rhs.cmp(count_lhs));
    for (nucleotide, count) in sortable_table {
        let percentage = *count as f32 * 100.0 / total_nucleotides;
        println!("{} {:.3}", nucleotide.to_str(nucleotide_len), percentage);
    }
    println!();
}

fn print_counts(nucleotide_str: &str, table: &NucleotideCounts) {
    let hash_key = NucleotideHash::from_str(nucleotide_str);
    if table.contains_key(&hash_key) {
        println!("{}\t{}", table[&hash_key], nucleotide_str);
    } else {
        println!("{}\t{}", 0, nucleotide_str);
    };
}
