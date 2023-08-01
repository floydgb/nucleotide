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
#[derive(Hash, Default, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Nucleotide {
    key: u64,
}

struct GenomeIter<'a> {
    nucleotide: Nucleotide,
    nucleotide_len: usize,
    bytes: Iter<'a, u8>,
}

type GenomeData = Arc<Vec<u8>>;
type NucleotideCounts = HashMap<Nucleotide, u32>;
type ThreadPool = Vec<thread::JoinHandle<(&'static str, NucleotideCounts)>>;

// Constants ------------------------------------------------------------------
const FILE_NAME: &str = "250000_in";
const FILE_START: &str = ">THREE";
const FILE_BUFFER_SIZE: usize = 65536;
const FILE_LINE_SIZE: usize = 64;
const NUCLEOTIDES: [char; 4] = ['A', 'C', 'T', 'G'];
const NUCLEOTIDE_STRS: [&str; 5] = #[rustfmt::skip] {
    ["GGTATTTTAATTTATAGT", "GGTATTTTAATT", "GGTATT", "GGTA", "GGT"]};

// Public Functions -----------------------------------------------------------
pub fn run() {
    let genome_arc = read_file();
    let worker_threads = par_count(&genome_arc);
    print_percentages(1, &genome_arc);
    print_percentages(2, &genome_arc);
    print_counts(worker_threads);
}

// Private Methods ------------------------------------------------------------
impl Nucleotide {
    fn push_byte(&mut self, byte: u8, nucleotide_len: usize) {
        self.key <<= 2;
        self.key |= ((byte >> 1) & 0b11) as u64;
        self.key &= (1u64 << (2 * nucleotide_len)) - 1;
    }

    fn to_str(&self, nucleotide_len: usize) -> String {
        let mut result = String::with_capacity(nucleotide_len);
        for i in (0..nucleotide_len).rev() {
            result.push(NUCLEOTIDES[((self.key >> (2 * i)) & 0b11) as usize]);
        }
        result
    }

    fn from(nucleotide_str: &str) -> Nucleotide {
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
fn read_file() -> GenomeData {
    let mut reader = BufReader::new(File::open(FILE_NAME).unwrap());
    let mut bytes = Vec::with_capacity(FILE_BUFFER_SIZE);
    let mut line = Vec::with_capacity(FILE_LINE_SIZE);
    let mut is_genome = false;
    while let Ok(b) = reader.read_until(b'\n', &mut line) {
        match (is_genome, b, line.starts_with(FILE_START.as_bytes())) {
            (true, 0, _) => break,
            (true, _, _) => bytes.extend_from_slice(&line[..b - 1]),
            (false, _, true) => is_genome = true,
            _ => (),
        }
        line.clear();
    }
    Arc::new(bytes)
}

fn build_iter(nucleotide_len: usize, genome: &GenomeData) -> GenomeIter {
    let mut bytes = genome.iter();
    let mut nucleotide = Nucleotide::default();
    for byte in bytes.by_ref().take(nucleotide_len - 1) {
        nucleotide.push_byte(*byte, nucleotide_len);
    }
    #[rustfmt::skip] {GenomeIter {nucleotide, nucleotide_len, bytes}}
}

fn count(nucleotide_len: usize, genome: &GenomeData) -> NucleotideCounts {
    let mut table = NucleotideCounts::default();
    for nucleotide in build_iter(nucleotide_len, genome) {
        *table.entry(nucleotide).or_insert(0) += 1;
    }
    table
}

fn print_percentages(nucleotide_len: usize, genome: &GenomeData) {
    let table = count(nucleotide_len, &genome);
    for (nucleotide, percentage) in calc_percentages(&table) {
        println!("{} {:.3}", nucleotide.to_str(nucleotide_len), percentage);
    }
    println!();
}

fn par_count(genome: &GenomeData) -> ThreadPool {
    NUCLEOTIDE_STRS
        .into_iter()
        .map(|nucleotide| {
            let genome = Arc::clone(genome);
            thread::spawn(move || {
                #[rustfmt::skip]
                {(nucleotide, count(nucleotide.len(), &genome))}
            })
        })
        .collect()
}

fn calc_percentages(table: &NucleotideCounts) -> Vec<(&Nucleotide, f32)> {
    let mut percent_table = Vec::default();
    let total_nucleotides: u32 = table.values().sum();
    for (nucleotide, count) in sort_by_count(table) {
        let percent = *count as f32 / total_nucleotides as f32 * 100_f32;
        percent_table.push((nucleotide, percent));
    }
    percent_table
}

fn sort_by_count(table: &NucleotideCounts) -> Vec<(&Nucleotide, &u32)> {
    let mut sorted_table: Vec<(&Nucleotide, &u32)> = table.iter().collect();
    sorted_table.sort_by(|(_, lhs), (_, rhs)| rhs.cmp(lhs));
    sorted_table
}

fn print_counts(worker_threads: ThreadPool) {
    for thread in worker_threads.into_iter().rev() {
        if let Ok((nucleotide, counts)) = thread.join() {
            let count = counts[&Nucleotide::from(nucleotide)];
            println!("{}\t{}", count, nucleotide);
        }
    }
}
