// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// contributed by Greg Floyd

// Based on k-nucleotide Rust #8
// Basic sanity & legibility

// Imports --------------------------------------------------------------------
#[rustfmt::skip] 
use {
    crate::seq,
    hashbrown::HashMap,
    std::{fs::File, io::{BufRead, BufReader}, slice::Iter, sync::Arc, thread}
};

// Types ----------------------------------------------------------------------
#[derive(Hash, Default, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Seq {
    hash_key: u64,
}

struct GenomeIter<'a> {
    seq_len: usize,
    needle: Seq,
    haystack: Iter<'a, u8>,
}

type Genome = Arc<Vec<u8>>;
type SeqCnts = HashMap<Seq, u32>;
type SeqCntsSort = Vec<(Seq, u32)>;
type SeqPcts = Vec<(Seq, f32)>;
type Thrds = Vec<thread::JoinHandle<(String, SeqCnts)>>;

// Public Functions -----------------------------------------------------------
#[rustfmt::skip]
pub fn run() {
    let seqs = seq!["GGT","GGTA","GGTATT","GGTATTTTAATT","GGTATTTTAATTTATAGT"];
    let genome = read_genome_file("250000_in");
    let threads = seq_cnts_par(seqs, &genome);
    println!("{}\n\n{}\n\n{}",
        show_all_seqs_len(1, &genome),
        show_all_seqs_len(2, &genome),
        show_seq_cnts_par(threads)
    );
}

// Private Methods ------------------------------------------------------------
impl Seq {
    fn push_byte(&mut self, byte: u8, seq_len: usize) {
        self.hash_key <<= 2;
        self.hash_key |= ((byte >> 1) & 0b11) as u64;
        self.hash_key &= (1u64 << (2 * seq_len)) - 1;
    }

    fn into(self, seq_len: usize) -> String {
        const NUCLEOTIDES: [char; 4] = ['A', 'C', 'T', 'G'];
        let mut str = String::with_capacity(seq_len);
        for i in (0..seq_len).rev() {
            str.push(NUCLEOTIDES[((self.hash_key >> (2 * i)) & 0b11) as usize]);
        }
        str
    }

    fn from(seq_str: &str) -> Seq {
        let mut seq = Seq::default();
        for byte in seq_str.as_bytes() {
            seq.push_byte(*byte, seq_str.len());
        }
        seq
    }
}

// Private Traits -------------------------------------------------------------
impl<'a> Iterator for GenomeIter<'a> {
    type Item = Seq;

    fn next(&mut self) -> Option<Seq> {
        self.haystack.next().map(|&byte| {
            self.needle.push_byte(byte, self.seq_len);
            self.needle
        })
    }
}

// Private Functions ----------------------------------------------------------
fn read_genome_file(file_name: &str) -> Genome {
    let mut buf = BufReader::new(File::open(file_name).expect("file found"));
    let (mut bytes, mut line) = (Vec::default(), Vec::default());
    let mut genome_start = false;
    while let Ok(b) = buf.read_until(b'\n', &mut line) {
        match (genome_start, b, line.starts_with(">THREE".as_bytes())) {
            (true, 0, _) => break,
            (true, _, _) => bytes.extend_from_slice(&line[..b - 1]),
            (false, _, true) => genome_start = true,
            _ => (),
        }
        line.clear();
    }
    Arc::new(bytes)
}

fn genome_iter(seq_len: usize, genome: &Genome) -> GenomeIter {
    let (mut haystack, mut needle) = (genome.iter(), Seq::default());
    for byte in haystack.by_ref().take(seq_len - 1) {
        needle.push_byte(*byte, seq_len);
    }
    #[rustfmt::skip] {GenomeIter {seq_len, needle, haystack}}
}

fn seq_cnt(g_iter: GenomeIter) -> SeqCnts {
    let mut seq_cnts = SeqCnts::default();
    for seq in g_iter {
        *seq_cnts.entry(seq).or_insert(0) += 1;
    }
    seq_cnts
}

fn seq_cnts_par(seq_strs: Vec<String>, genome: &Genome) -> Thrds {
    Iterator::collect(seq_strs.into_iter().map(|str| {
        let (seq_len, genome) = (str.len(), Arc::clone(genome));
        thread::spawn(move || (str, seq_cnt(genome_iter(seq_len, &genome))))
    }))
}

fn sort_by_cnt(seq_cnts: SeqCnts) -> SeqCntsSort {
    let mut seq_cnts_sort: SeqCntsSort = seq_cnts.into_iter().collect();
    seq_cnts_sort.sort_by(|(_, l_cnt), (_, r_cnt)| r_cnt.cmp(l_cnt));
    seq_cnts_sort
}

fn calc_pcts(seq_cnts: SeqCnts) -> SeqPcts {
    let tot_seqs: u32 = seq_cnts.values().sum();
    let mut seq_pcts = Vec::default();
    for (seq, cnt) in sort_by_cnt(seq_cnts) {
        seq_pcts.push((seq, cnt as f32 / tot_seqs as f32 * 100_f32));
    }
    seq_pcts
}

fn show_all_seqs_len(seq_len: usize, genome: &Genome) -> String {
    let mut str = Vec::default();
    for (seq, pct) in calc_pcts(seq_cnt(genome_iter(seq_len, genome))) {
        str.push(format!("{} {:.3}", seq.into(seq_len), pct));
    }
    str.join("\n")
}

fn show_seq_cnts_par(pool: Thrds) -> String {
    let mut str = Vec::default();
    for thrd in pool {
        let (seq_str, seq_cnts) = thrd.join().expect("thread halts");
        let count = seq_cnts.get(&Seq::from(&seq_str)).unwrap_or(&0);
        str.push(format!("{}\t{}", count, seq_str));
    }
    str.join("\n")
}
