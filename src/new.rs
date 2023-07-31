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

struct Genome<'a> {
    occurance: Nucleotide,
    cursor: Iter<'a, u8>,
    cursor_size: usize,
}

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Nucleotide {
    code: u64,
}

// Constants ------------------------------------------------------------------
const FILE_NAME: &str = "250000_in";
const FILE_START: &str = ">THREE";
const MAX_BUFFER_SIZE: usize = 65536;
const MAX_LINE_SIZE: usize = 80;
const NUCLEOTIDES: [&str; 5] = [
    "GGTATTTTAATTTATAGT",
    "GGTATTTTAATT",
    "GGTATT",
    "GGTA",
    "GGT",
];
const A_CODE: u8 = 0;
const C_CODE: u8 = 1;
const T_CODE: u8 = 2;
const G_CODE: u8 = 3;

// Methods --------------------------------------------------------------------
impl Nucleotide {
    fn push(&mut self, byte: u8, cursor_size: usize) {
        self.code <<= 2;
        self.code |= byte as u64;
        self.code &= (1u64 << (2 * cursor_size)) - 1;
    }

    fn to_string(self, cursor_size: usize) -> String {
        let mut result = String::default();
        let mut code = self.code as u8;
        for _ in 0..cursor_size {
            let c = match code & 0b11 {
                A_CODE => b'A',
                T_CODE => b'T',
                G_CODE => b'G',
                C_CODE => b'C',
                _ => unreachable!(),
            };
            result.push(c.into());
            code >>= 2;
        }
        result.chars().rev().collect()
    }
}

// Traits ---------------------------------------------------------------------
impl<'a> Iterator for Genome<'a> {
    type Item = Nucleotide;

    fn next(&mut self) -> Option<Nucleotide> {
        self.cursor.next().map(|&byte| {
            self.occurance.push(byte, self.cursor_size);
            self.occurance
        })
    }
}

// Functions ------------------------------------------------------------------
#[inline(always)]
fn encode_byte(c: &u8) -> u8 {
    (c >> 1) & 0b11
}

fn new_genome(cursor_size: usize, bytes: &[u8]) -> Genome {
    let mut iter = bytes.iter();
    let mut code = Nucleotide { code: 0 };
    for c in iter.by_ref().take(cursor_size - 1) {
        code.push(*c, cursor_size);
    }
    Genome {
        cursor: iter,
        occurance: code,
        cursor_size,
    }
}

fn from_str(nucleotide: &str) -> Nucleotide {
    let mut result = Nucleotide { code: 0 };
    for byte in nucleotide.as_bytes() {
        result.push(encode_byte(byte), nucleotide.len());
    }
    result
}

fn read_file() -> Vec<u8> {
    let mut reader = BufReader::new(File::open(FILE_NAME).unwrap());
    let mut result = Vec::with_capacity(MAX_BUFFER_SIZE);
    let mut line = String::with_capacity(MAX_LINE_SIZE);
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
        result.extend(bytes[..bytes.len() - 1].iter().map(encode_byte));
        line.clear();
    }
    result
}

fn count_occurances(frame_size: usize, genome: &[u8]) -> NucleotideCounts {
    let mut counts = NucleotideCounts::default();
    for code in new_genome(frame_size, genome) {
        *counts.entry(code).or_insert(0) += 1;
    }
    counts
}

fn print_percents(frame_size: usize, genome: &[u8]) {
    let hmap = &count_occurances(frame_size, &genome);
    let total = hmap.values().sum::<u32>() as f32;
    let mut v: Vec<_> = hmap.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for &(code, count) in v.iter() {
        let freq_percent = *count as f32 * 100.0 / total;
        println!("{} {:.3}", code.to_string(frame_size), freq_percent);
    }
    println!();
}

fn print_counts(occurance: &str, counts: &NucleotideCounts) {
    let code = from_str(occurance);
    if counts.contains_key(&code) {
        println!("{}\t{}", counts[&code], occurance);
    } else {
        println!("{}\t{}", 0, occurance);
    };
}

pub fn run() {
    let genome = Arc::new(read_file());

    let workers: Vec<_> = NUCLEOTIDES
        .into_iter()
        .map(|occurance| {
            let genome = genome.clone();
            thread::spawn(move || (occurance, count_occurances(occurance.len(), &genome)))
        })
        .collect();

    print_percents(1, &genome);
    print_percents(2, &genome);
    for t in workers.into_iter().rev() {
        if let Ok((occurance, counts)) = t.join() {
            print_counts(occurance, &counts);
        }
    }
}
