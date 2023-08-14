// The Computer Language Benchmarks Game
// https://programming-language-benchmarks.vercel.app/problem/knucleotide
//
// contributed by Greg Floyd

// Imports --------------------------------------------------------------------
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::io::{BufRead, BufReader};
use std::{fs::File, slice, sync::Arc, thread::spawn, thread::JoinHandle};
use {crate::str, hashbrown::HashMap};

// Constants ------------------------------------------------------------------
const NUCLEOTIDES: [char; 4] = ['A', 'C', 'T', 'G'];
const NUM_CHUNKS: usize = 64;
const START: &[u8] = ">TH".as_bytes();
const FILE: usize = 15000000;

// Types ----------------------------------------------------------------------
#[derive(Hash, Default, PartialEq, Eq, Clone, Copy)]
pub struct Seq {
    hash_key: u64,
}

pub struct KNucleotides<'a> {
    seq_len: usize,
    cur_seq: Seq,
    genome_iter: slice::Iter<'a, u8>,
}

pub type Threads = Vec<JoinHandle<(String, usize)>>;

// Public Functions -----------------------------------------------------------
#[rustfmt::skip]
pub fn main() {
    let genome = read_file("2500000_in");
    let seqs = str!["GGT","GGTATTTTAATT","GGTA","GGTATTTTAATTTATAGT","GGTATT"];

    let counts = count(seqs, &genome);
    let (k1, k2) = (count_k(1, &genome), count_k(2, &genome));

    println!("{}\n\n{}\n\n{}", show_k(1, k1), show_k(2, k2), show(counts));
}

pub fn read_file(path: &str) -> Arc<Vec<u8>> {
    let (mut read, mut r) = (false, BufReader::new(File::open(path).unwrap()));
    let (mut buf, mut l) = (Vec::with_capacity(FILE), Vec::with_capacity(64));
    while r.read_until(b'\n', &mut l).unwrap_or(0) > 0 {
        match read {
            true => buf.extend_from_slice(&l[..l.len() - 1]),
            false => read = l.starts_with(START),
        }
        l.clear();
    }
    Arc::new(buf)
}

pub fn count(seq_strs: Vec<String>, genome: &Arc<Vec<u8>>) -> Threads {
    let mut pool = Vec::with_capacity(seq_strs.len());
    for seq_str in sort_by_len(seq_strs) {
        let arc = Arc::clone(&genome);
        pool.push(spawn(move || par_count(&seq_str, seq_str.len(), &arc)));
    }
    pool
}

pub fn count_k(k: usize, genome: &[u8]) -> HashMap<Seq, u32> {
    chunks(genome.len() / NUM_CHUNKS, k - 1, genome)
        .into_par_iter()
        .map(|chunk| inner_count_k(k, chunk))
        .reduce(HashMap::default, merge)
}

pub fn show(counts: Threads) -> String {
    let mut str = Vec::with_capacity(counts.len());
    for thrd in counts {
        let (seq_str, seq_cnt) = thrd.join().expect("thread halts");
        str.push(format!("{}\t{}", seq_cnt, seq_str));
    }
    str.join("\n")
}

pub fn show_k(k: usize, k_counts: HashMap<Seq, u32>) -> String {
    let mut str = Vec::with_capacity(k_counts.len());
    for (seq, pct) in calc_percents(k_counts) {
        str.push(format!("{} {:.3}", seq.to_str(k), pct));
    }
    str.join("\n")
}

// Traits ---------------------------------------------------------------------
impl<'a> Iterator for KNucleotides<'a> {
    type Item = Seq;
    fn next(&mut self) -> Option<Seq> {
        self.genome_iter.next().map(|&byte| {
            self.cur_seq.push(byte, self.seq_len);
            self.cur_seq
        })
    }
}

// Private Functions ----------------------------------------------------------
impl Seq {
    fn push(&mut self, byte: u8, seq_len: usize) {
        self.hash_key =
            ((self.hash_key << 2) | ((byte >> 1) & 0b11) as u64) & ((1u64 << (2 * seq_len)) - 1);
    }

    fn to_str(self, seq_len: usize) -> String {
        let mut str = String::with_capacity(seq_len);
        for seq_i in (0..seq_len).rev() {
            let str_i = ((self.hash_key >> (2 * seq_i)) & 0b11) as usize;
            str.push(NUCLEOTIDES[str_i]);
        }
        str
    }

    fn from_str(seq_str: &str) -> Self {
        let mut seq = Self::default();
        for byte in seq_str.as_bytes() {
            seq.push(*byte, seq_str.len());
        }
        seq
    }
}

fn k_nucleotides(seq_len: usize, genome: &[u8]) -> KNucleotides {
    KNucleotides {
        seq_len,
        cur_seq: Seq::default(),
        genome_iter: genome.into_iter(),
    }
}

fn chunks(chunk_size: usize, overlap: usize, bytes: &[u8]) -> Vec<&[u8]> {
    bytes
        .windows(chunk_size + overlap)
        .step_by(chunk_size)
        .collect()
}

fn par_count(seq_str: &str, seq_len: usize, genome: &[u8]) -> (String, usize) {
    let count = chunks(genome.len() / NUM_CHUNKS, seq_len - 1, genome)
        .into_par_iter()
        .map(|chunk| inner_count(Seq::from_str(seq_str), seq_len, chunk))
        .sum();
    (seq_str.into(), count)
}

fn inner_count(target_seq: Seq, seq_len: usize, genome: &[u8]) -> usize {
    k_nucleotides(seq_len, genome)
        .into_iter()
        .filter(|&seq| seq == target_seq)
        .count()
}

fn inner_count_k(seq_len: usize, genome: &[u8]) -> HashMap<Seq, u32> {
    let capacity = NUCLEOTIDES.len().pow(seq_len as u32);
    let mut seq_cnts = HashMap::<Seq, u32>::with_capacity(capacity);
    for seq in k_nucleotides(seq_len, genome).into_iter() {
        *seq_cnts.entry(seq).or_insert(0) += 1;
    }
    seq_cnts
}

fn calc_percents(seq_cnts: HashMap<Seq, u32>) -> Vec<(Seq, f32)> {
    let tot_seqs: u32 = seq_cnts.values().sum();
    let mut pcts = Vec::with_capacity(seq_cnts.len());
    for (seq, cnt) in sort_by_count(seq_cnts) {
        pcts.push((seq, (cnt * 100) as f32 / tot_seqs as f32));
    }
    pcts
}

fn sort_by_len(seq_strs: Vec<String>) -> Vec<String> {
    let mut seq_strs = seq_strs;
    seq_strs.sort_by(|l, r| l.len().cmp(&r.len()));
    seq_strs
}

fn sort_by_count(seq_cnts: HashMap<Seq, u32>) -> Vec<(Seq, u32)> {
    let mut seq_cnts_sort: Vec<_> = seq_cnts.into_iter().collect();
    seq_cnts_sort.sort_by(|(_, l_cnt), (_, r_cnt)| r_cnt.cmp(l_cnt));
    seq_cnts_sort
}

fn merge(mut a: HashMap<Seq, u32>, b: HashMap<Seq, u32>) -> HashMap<Seq, u32> {
    for (seq, cnt) in b {
        *a.entry(seq).or_insert(0) += cnt;
    }
    a
}
