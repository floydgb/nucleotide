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
struct SeqHash {
    key: u64,
}

struct GenomeIter<'a> {
    seq: SeqHash,
    k_len: usize,
    bytes: Iter<'a, u8>,
}

type GenomeData = Arc<Vec<u8>>;
type SeqCounts = HashMap<SeqHash, u32>;
type SortedSeqCounts = Vec<(SeqHash, u32)>;
type SeqPercents = Vec<(SeqHash, f32)>;
type ThreadPool = Vec<thread::JoinHandle<(&'static str, SeqCounts)>>;

// Constants ------------------------------------------------------------------
const FILE_NAME: &str = "250000_in";
const FILE_START: &str = ">THREE";
const FILE_BUFFER_SIZE: usize = 65536;
const FILE_LINE_SIZE: usize = 64;
const NUCLEOTIDES: [char; 4] = ['A', 'C', 'T', 'G'];
const SEQS: [&str; 5] = #[rustfmt::skip] {
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
impl SeqHash {
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

    fn from(seq_str: &str) -> SeqHash {
        let mut seq = SeqHash::default();
        for byte in seq_str.as_bytes() {
            seq.push_byte(*byte, seq_str.len());
        }
        seq
    }
}

// Private Traits -------------------------------------------------------------
impl<'a> Iterator for GenomeIter<'a> {
    type Item = SeqHash;

    fn next(&mut self) -> Option<SeqHash> {
        self.bytes.next().map(|&byte| {
            self.seq.push_byte(byte, self.k_len);
            self.seq
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
    let mut seq = SeqHash::default();
    for byte in bytes.by_ref().take(k_len - 1) {
        seq.push_byte(*byte, k_len);
    } 
    GenomeIter {seq, k_len, bytes}}
}

fn count(k_len: usize, genome: &GenomeData) -> SeqCounts {
    let mut seq_counts = SeqCounts::default();
    for seq in build_iter(k_len, genome) {
        *seq_counts.entry(seq).or_insert(0) += 1;
    }
    seq_counts
}

fn par_count(genome: &GenomeData) -> ThreadPool {
    Iterator::collect(SEQS.into_iter().map(|seq_str| {
        let genome = Arc::clone(genome);
        thread::spawn(move || (seq_str, count(seq_str.len(), &genome)))
    }))
}

fn sort_by_count(seq_counts: SeqCounts) -> SortedSeqCounts {
    let mut sorted_seqs: SortedSeqCounts = seq_counts.into_iter().collect();
    sorted_seqs.sort_by(|(_, lhs), (_, rhs)| rhs.cmp(lhs));
    sorted_seqs
}

fn calc_percentages(seq_counts: SeqCounts) -> SeqPercents {
    let mut seq_percents = Vec::default();
    let total_seqs: u32 = seq_counts.values().sum();
    for (seq, count) in sort_by_count(seq_counts) {
        let percent = count as f32 / total_seqs as f32 * 100_f32;
        seq_percents.push((seq, percent));
    }
    seq_percents
}

fn show_percents(k_len: usize, genome: &GenomeData) -> String {
    #[rustfmt::skip] { calc_percentages(count(k_len, &genome)).iter()
        .map(|(seq, percentage)| {
            format!("{} {:.3}", seq.to_str(k_len), percentage)
        }).collect::<Vec<String>>().join("\n")}
}

fn show_counts(threads: ThreadPool) -> String {
    #[rustfmt::skip] { threads.into_iter().rev()
        .map(|thread| { 
            let (seq_str, counts) = thread.join().expect("threads halt"); 
            format!("{}\t{}", counts[&SeqHash::from(seq_str)], seq_str)  
        }).collect::<Vec<String>>().join("\n")}
}
