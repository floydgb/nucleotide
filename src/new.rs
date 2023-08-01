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
type NucleotideCounts = HashMap<Nucleotide, u32>;

struct GenomeIter<'a> {
    nucleotide: Nucleotide,
    nucleotide_len: usize,
    bytes: Iter<'a, u8>,
}

#[derive(Hash, Default, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Nucleotide {
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
    let genome = Arc::new(read_genome_file());
    let worker_threads: Vec<_> = NUCLEOTIDE_STRS
        .into_iter()
        .map(|nucleotide| {
            let genome = genome.clone();
            thread::spawn(move || (nucleotide, count_nucleotides(nucleotide.len(), &genome)))
        })
        .collect();
    for (nucleotide, percentage) in calc_percentages(&count_nucleotides(1, &genome)) {
        println!("{} {:.3}", nucleotide.to_str(1), percentage);
    }
    println!();
    for (nucleotide, percentage) in calc_percentages(&count_nucleotides(2, &genome)) {
        println!("{} {:.3}", nucleotide.to_str(2), percentage);
    }
    println!();
    for thread in worker_threads.into_iter().rev() {
        if let Ok((nucleotide, counts)) = thread.join() {
            println!("{}\t{}", counts[&Nucleotide::from(nucleotide)], nucleotide);
        }
    }
}

// Private Methods ------------------------------------------------------------
// todo: keep track of its own length
impl Nucleotide {
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

    fn from(nucleotide_str: &str) -> Self {
        let mut nucleotide = Nucleotide::default();
        for byte in nucleotide_str.as_bytes() {
            nucleotide.push_byte(*byte, nucleotide_str.len());
        }
        nucleotide
    }
}

// Private Traits -------------------------------------------------------------
impl<'a> Iterator for GenomeIter<'a> {
    type Item = Nucleotide;

    fn next(&mut self) -> Option<Nucleotide> {
        self.bytes.next().map(|&byte| {
            self.nucleotide.push_byte(byte, self.nucleotide_len);
            self.nucleotide
        })
    }
}

// Private Functions ----------------------------------------------------------
fn read_genome_file() -> Vec<u8> {
    let mut reader = BufReader::new(File::open(FILE_NAME).unwrap());
    let mut genome_bytes = Vec::with_capacity(FILE_BUFFER_SIZE);
    let mut line = String::with_capacity(FILE_LINE_SIZE);
    let mut is_genome = false;
    while let Ok(b) = reader.read_line(&mut line) {
        match (is_genome, b) {
            (false, _) if line.starts_with(FILE_START) => is_genome = true,
            (false, _) => {}
            (true, 0) => break,
            (true, _) => genome_bytes.extend(line.as_bytes()[..line.as_bytes().len() - 1].iter()),
        }
        line.clear();
    }
    genome_bytes
}

fn build_genome_iter(nucleotide_len: usize, genome: &[u8]) -> GenomeIter {
    let mut bytes = genome.iter();
    let mut nucleotide = Nucleotide::default();
    for byte in bytes.by_ref().take(nucleotide_len - 1) {
        nucleotide.push_byte(*byte, nucleotide_len);
    }
    #[rustfmt::skip] { GenomeIter {nucleotide, nucleotide_len, bytes} }
}

//todo: pass in genome_iter
fn count_nucleotides(nucleotide_len: usize, genome: &[u8]) -> NucleotideCounts {
    let mut count_table = NucleotideCounts::default();
    for nucleotide in build_genome_iter(nucleotide_len, genome) {
        *count_table.entry(nucleotide).or_insert(0) += 1;
    }
    count_table
}

fn calc_percentages(count_table: &NucleotideCounts) -> Vec<(&Nucleotide, f32)> {
    let mut percent_table = Vec::new();
    let total_nucleotides = count_table.values().sum::<u32>();
    for (nucleotide, count) in sort_nucleotide_by_count(count_table) {
        let percent = *count as f32 / total_nucleotides as f32 * 100_f32;
        percent_table.push((nucleotide, percent));
    }
    percent_table
}

fn sort_nucleotide_by_count(count_table: &NucleotideCounts) -> Vec<(&Nucleotide, &u32)> {
    let mut sorted_table: Vec<_> = count_table.iter().collect();
    sorted_table.sort_by(|(_, count_lhs), (_, count_rhs)| count_rhs.cmp(count_lhs));
    sorted_table
}
