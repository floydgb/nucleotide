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
pub struct Seq {
    key: u64,
}

pub struct KNucleotides<'a> {
    k: usize,
    seq: Seq,
    gen: Iter<'a, u8>,
}

pub type Threads = Vec<JoinHandle<(String, usize)>>;

// Public Functions -----------------------------------------------------------
#[rustfmt::skip] pub fn main() {
    let genome = read_file("2500000_in"); 
    let seqs = str!["GGT","GGTATTTTAATT","GGTA","GGTATTTTAATTTATAGT","GGTATT"];
    let counts = count(seqs, &genome);
    let (k1, k2) = (count_k(1, &genome), count_k(2, &genome));
    println!("{}\n\n{}\n\n{}", show_k(1, k1), show_k(2, k2), show(counts));
}

pub fn read_file(path: &str) -> Arc<Vec<u8>> {
    let (mut read, mut r) = (false, BufReader::new(File::open(path).unwrap()));
    let (mut buf, mut l) = (Vec::with_capacity(15000000), Vec::new());
    #[rustfmt::skip] while r.read_until(b'\n', &mut l).unwrap_or(0) > 0 {
        if read { buf.extend_from_slice(&l[..l.len() - 1]) } 
        else { read = l.starts_with(">TH".as_bytes()) }
        l.clear();
    }
    Arc::new(buf)
}

pub fn count(seqs: Vec<String>, genome: &Arc<Vec<u8>>) -> Threads {
    let mut threads = Vec::with_capacity(seqs.len());
    sort_len(seqs).for_each(|seq_str| {
        let arc = Arc::clone(&genome);
        threads.push(spawn(move || par_count(&seq_str, seq_str.len(), &arc)));
    });
    threads
}

pub fn count_k(k: usize, genome: &[u8]) -> HashMap<Seq, u32> {
    #[rustfmt::skip] chunks(genome.len() / 64, k - 1, genome)
        .map(|chunk| inner_count_k(k, chunk)).reduce(HashMap::default, merge)
}

pub fn show(counts: Threads) -> String {
    let mut str = Vec::with_capacity(counts.len());
    counts.into_iter().for_each(|thrd| {
        let (seq_str, seq_cnt) = thrd.join().expect("thread halts");
        str.push(format!("{}\t{}", seq_cnt, seq_str));
    });
    str.join("\n")
}

pub fn show_k(k: usize, c: HashMap<Seq, u32>) -> String {
    let mut str = Vec::with_capacity(c.len());
    calc_percents(c.values().sum(), c)
        .for_each(|(s, p)| str.push(format!("{} {:.3}", s.to_str(k), p)));
    str.join("\n")
}

// Traits ---------------------------------------------------------------------
impl<'a> Iterator for KNucleotides<'a> {
    type Item = Seq; #[rustfmt::skip]
    fn next(&mut self) -> Option<Seq> {
        self.gen.next().map(|&byte| {self.seq.push(byte, self.k); self.seq})
    }
}

// Private Functions ----------------------------------------------------------
impl Seq {
    fn push(&mut self, byte: u8, k: usize) {
        self.key = (self.key << 2) | ((byte >> 1) & 0b11) as u64;
        self.key &= (1u64 << (2 * k)) - 1;
    }

    fn to_str(self, k: usize) -> String {
        let mut s = String::with_capacity(k);
        #[rustfmt::skip] (0..k).rev().for_each(|i|
        s.push(['A', 'C', 'T', 'G'][((self.key >> (2 * i)) & 0b11) as usize]));
        s
    }

    fn from_str(s: &str) -> Self {
        let mut seq = Self::default();
        s.as_bytes().into_iter().for_each(|b| seq.push(*b, s.len()));
        seq
    }
}

fn k_nucl(k: usize, g: &[u8]) -> KNucleotides {
    #[rustfmt::skip]KNucleotides {k, seq: Seq::default(), gen: g.iter()}
}

fn chunks(s: usize, o: usize, bs: &[u8]) -> rayon::vec::IntoIter<&[u8]> {
    #[rustfmt::skip]bs.windows(s + o).step_by(s)
        .collect::<Vec<&[u8]>>().into_par_iter()
}

fn par_count(seq: &str, k: usize, genome: &[u8]) -> (String, usize) {
    #[rustfmt::skip](seq.into(), chunks(genome.len() / 64, k-1, genome)
        .map(|chunk| inner_count(Seq::from_str(seq), k, chunk)).sum())
}

fn inner_count(seq: Seq, k: usize, gen: &[u8]) -> usize {
    k_nucl(k, gen).filter(|&s| s == seq).count()
}

fn inner_count_k(k: usize, genome: &[u8]) -> HashMap<Seq, u32> {
    let mut seq_cnts = HashMap::with_capacity(4usize.pow(k as u32));
    k_nucl(k, genome).for_each(|seq| *seq_cnts.entry(seq).or_insert(0) += 1);
    seq_cnts
}

fn calc_percents(tot: u32, cnt: HashMap<Seq, u32>) -> IntoIter<(Seq, f32)> {
    let mut p = Vec::with_capacity(cnt.len());
    sort_cnt(cnt).for_each(|(s, c)| p.push((s, c as f32 * 100. / tot as f32)));
    p.into_iter()
}

fn sort_len(mut seqs: Vec<String>) -> IntoIter<String> {
    seqs.sort_by(|l, r| l.len().cmp(&r.len()));
    seqs.into_iter()
}

fn sort_cnt(counts: HashMap<Seq, u32>) -> IntoIter<(Seq, u32)> {
    let mut counts_sorted: Vec<(Seq, u32)> = counts.into_iter().collect();
    counts_sorted.sort_by(|(_, l_cnt), (_, r_cnt)| r_cnt.cmp(l_cnt));
    counts_sorted.into_iter()
}

fn merge(mut a: HashMap<Seq, u32>, b: HashMap<Seq, u32>) -> HashMap<Seq, u32> {
    b.iter().for_each(|(s, c)| *a.entry(*s).or_insert(0) += c);
    a
}
