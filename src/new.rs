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
    k_len: usize,
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
    let genome = read_file(FILE_NAME);
    let worker_threads = par_count(&genome);
    println!(
        "{}\n\n{}\n\n{}",
        show_percents(1, &genome),
        show_percents(2, &genome),
        show_counts(worker_threads)
    );
}

// Private Methods ------------------------------------------------------------
impl Nucleotide {
    fn push_byte(&mut self, byte: u8, k_len: usize) {
        self.key <<= 2;
        self.key |= ((byte >> 1) & 0b11) as u64;
        self.key &= (1u64 << (2 * k_len)) - 1;
    }

    fn to_str(&self, k_len: usize) -> String {
        let mut result = String::with_capacity(k_len);
        for i in (0..k_len).rev() {
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
            self.nucleotide.push_byte(byte, self.k_len);
            self.nucleotide
        })
    }
}

// Private Functions ----------------------------------------------------------
fn read_file(file_name: &str) -> GenomeData {
    let mut buf = BufReader::new(File::open(file_name).expect("file found"));
    let mut bytes = Vec::with_capacity(FILE_BUFFER_SIZE);
    let mut line = Vec::with_capacity(FILE_LINE_SIZE);
    let mut is_genome = false;
    while let Ok(b) = buf.read_until(b'\n', &mut line) {
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

fn build_iter(k_len: usize, genome: &GenomeData) -> GenomeIter {
    #[rustfmt::skip] { let mut bytes = genome.iter();
    let mut nucleotide = Nucleotide::default();
    for byte in bytes.by_ref().take(k_len - 1) {
        nucleotide.push_byte(*byte, k_len);
    } 
    GenomeIter {nucleotide, k_len, bytes}}
}

fn count(k_len: usize, genome: &GenomeData) -> NucleotideCounts {
    let mut table = NucleotideCounts::default();
    for nucleotide in build_iter(k_len, genome) {
        *table.entry(nucleotide).or_insert(0) += 1;
    }
    table
}

fn par_count(genome: &GenomeData) -> ThreadPool {
    Iterator::collect(NUCLEOTIDE_STRS.into_iter().map(|nucleotide| {
        let genome = Arc::clone(genome);
        thread::spawn(move || (nucleotide, count(nucleotide.len(), &genome)))
    }))
}

fn sort_by_count(table: &NucleotideCounts) -> Vec<(&Nucleotide, &u32)> {
    let mut sorted_table: Vec<_> = table.iter().collect();
    sorted_table.sort_by(|(_, lhs), (_, rhs)| rhs.cmp(lhs));
    sorted_table
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

fn show_percents(k_len: usize, genome: &GenomeData) -> String {
    #[rustfmt::skip] { calc_percentages(&count(k_len, &genome)).iter()
        .map(|(nucleotide, percentage)| {
            format!("{} {:.3}", nucleotide.to_str(k_len), percentage)})
        .collect::<Vec<String>>().join("\n")}
}

fn show_counts(threads: ThreadPool) -> String {
    #[rustfmt::skip] { threads.into_iter().rev()
        .map(|thread| { match thread.join().expect("threads halt") {
            (nucleotide, counts) => {
                let count = counts[&Nucleotide::from(nucleotide)];
                format!("{}\t{}", count, nucleotide).into()}}})
        .collect::<Vec<String>>().join("\n")}
}
