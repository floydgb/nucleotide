// The Computer Language Benchmarks Game
// https://programming-language-benchmarks.vercel.app/problem/knucleotide
//
// contributed by Greg Floyd

// Imports --------------------------------------------------------------------
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::io::{BufRead, BufReader};
use std::{fs::File, slice::Iter, sync::Arc, thread::spawn, thread::JoinHandle};
use {crate::str, hashbrown::HashMap, std::vec::IntoIter};

// Types ----------------------------------------------------------------------
#[derive(Hash, Default, PartialEq, Eq, Clone, Copy)]
pub struct Sequence {
    key: u64,
}
pub struct KGenomeIter<'a> {
    k: usize,
    seq: Sequence,
    genome: Iter<'a, u8>,
}
pub type SeqCounts = HashMap<Sequence, u32>;
pub type Threads = Vec<JoinHandle<(String, usize)>>;

// Main -----------------------------------------------------------------------
#[rustfmt::skip]
pub fn main() {
    let genome = read_file("2500000_in"); 
    let seqs = str!["GGT","GGTATTTTAATT","GGTA","GGTATTTTAATTTATAGT","GGTATT"];

    let seq_counts = count(seqs, &genome);
    let (k1, k2) = (count_k(1, &genome), count_k(2, &genome));

    println!("{}\n\n{}\n\n{}", show_k(1, k1), show_k(2, k2), show(seq_counts));
}

// Public Functions -----------------------------------------------------------
pub fn read_file(path: &str) -> Arc<Vec<u8>> {
    let (mut read, mut r) = (false, BufReader::new(File::open(path).unwrap()));
    let (mut buf, mut line) = (Vec::with_capacity(15000000), Vec::new());
    while r.read_until(b'\n', &mut line).unwrap_or(0) > 0 {
        if read {
            buf.extend_from_slice(&line[..line.len() - 1])
        } else {
            read = line.starts_with(">TH".as_bytes())
        }
        line.clear();
    }
    Arc::new(buf)
}

pub fn count(seqs: Vec<String>, genome: &Arc<Vec<u8>>) -> Threads {
    let mut threads = Vec::with_capacity(seqs.len());
    for str in sort_len(seqs) {
        let arc = Arc::clone(&genome);
        threads.push(spawn(move || par_count(&str, &arc)));
    }
    threads
}

pub fn count_k(k: usize, genome: &[u8]) -> SeqCounts {
    chunks(genome.len() / 64, k - 1, genome)
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

pub fn show_k(k: usize, counts: SeqCounts) -> String {
    let mut str = Vec::with_capacity(counts.len());
    for (s, p) in calc_percents(counts.values().sum(), counts) {
        str.push(format!("{} {:.3}", s.to_str(k), p))
    }
    str.join("\n")
}

// Private Functions ----------------------------------------------------------
impl Sequence {
    fn push(&mut self, byte: u8, k: usize) {
        self.key = (self.key << 2) | ((byte >> 1) & 0b11) as u64;
        self.key &= (1u64 << (2 * k)) - 1;
    }

    fn to_str(self, k: usize) -> String {
        let mut s = String::with_capacity(k);
        for i in (0..k).rev() {
            s.push(['A', 'C', 'T', 'G'][(self.key >> (2 * i) & 0b11) as usize])
        }
        s
    }

    fn from_str(s: &str) -> Self {
        let mut seq = Self::default();
        for b in s.as_bytes().into_iter() {
            seq.push(*b, s.len())
        }
        seq
    }
}

impl<'a> Iterator for KGenomeIter<'a> {
    type Item = Sequence;

    fn next(&mut self) -> Option<Sequence> {
        self.seq.push(*self.genome.next()?, self.k);
        Some(self.seq)
    }
}

#[rustfmt::skip]
fn k_genome_iter(k: usize, genome: &[u8]) -> KGenomeIter {
   KGenomeIter {k, seq: Sequence::default(), genome: genome.into_iter()}
}

fn chunks(len: usize, overlap: usize, genome: &[u8]) -> Vec<&[u8]> {
    genome.windows(len + overlap).step_by(len).collect()
}

fn par_count(seq: &str, genome: &[u8]) -> (String, usize) {
    let count = chunks(genome.len() / 64, seq.len() - 1, genome)
        .into_par_iter()
        .map(|chunk| inner_count(Sequence::from_str(seq), seq.len(), chunk))
        .sum();
    (seq.into(), count)
}

fn inner_count(seq: Sequence, k: usize, genome: &[u8]) -> usize {
    k_genome_iter(k, genome).filter(|&s| s == seq).count()
}

fn inner_count_k(k: usize, genome: &[u8]) -> SeqCounts {
    let mut counts = HashMap::with_capacity(4usize.pow(k as u32));
    for seq in k_genome_iter(k, genome) {
        *counts.entry(seq).or_insert(0) += 1
    }
    counts
}

fn calc_percents(total: u32, counts: SeqCounts) -> IntoIter<(Sequence, f32)> {
    let mut percents = Vec::with_capacity(counts.len());
    for (seq, count) in sort_cnt(counts) {
        percents.push((seq, count as f32 * 100. / total as f32))
    }
    percents.into_iter()
}

fn sort_len(mut seqs: Vec<String>) -> IntoIter<String> {
    seqs.sort_by(|l, r| l.len().cmp(&r.len()));
    seqs.into_iter()
}

fn sort_cnt(counts: SeqCounts) -> IntoIter<(Sequence, u32)> {
    let mut counts: Vec<(Sequence, u32)> = counts.into_iter().collect();
    counts.sort_by(|(_, l_cnt), (_, r_cnt)| r_cnt.cmp(l_cnt));
    counts.into_iter()
}

fn merge(mut l_counts: SeqCounts, r_counts: SeqCounts) -> SeqCounts {
    for (seq, count) in r_counts.iter() {
        *l_counts.entry(*seq).or_insert(0) += count
    }
    l_counts
}
