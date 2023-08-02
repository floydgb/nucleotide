// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// contributed by Greg Floyd

// Based on k-nucleotide Rust #8
// Basic sanity & legibility

// Imports --------------------------------------------------------------------
use {
    hashbrown::HashMap,
    std::{
        fs::File,
        io::{BufRead, BufReader},
        slice::Iter,
        sync::Arc,
        thread::{spawn, JoinHandle},
    },
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
type ThrdPool = Vec<JoinHandle<(&'static str, SeqCnts)>>;

// Constants ------------------------------------------------------------------
const FILE_NAME: &str = "250000_in";
const FILE_START: &str = ">THREE";
const FILE_BUFFER_SIZE: usize = 65536;
const FILE_LINE_SIZE: usize = 64;
const DECIMALS: usize = 3;
const SEQ_STRS: [&str; 5] = #[rustfmt::skip] {
        ["GGTATTTTAATTTATAGT", "GGTATTTTAATT", "GGTATT", "GGTA", "GGT"]};

// Public Functions -----------------------------------------------------------
pub fn run() {
    let genome = read_file(FILE_NAME);
    let worker_thrds = par_seq_cnt(&genome);
    print!(
        "{}\n\n{}\n\n{}\n",
        show_seq_pcts(genome_iter(1, &genome)),
        show_seq_pcts(genome_iter(2, &genome)),
        show_seq_cnts(worker_thrds)
    );
}

// Private Methods ------------------------------------------------------------
impl Seq {
    fn push_byte(&mut self, byte: u8, seq_len: usize) {
        self.hash_key <<= 2;
        self.hash_key |= ((byte >> 1) & 0b11) as u64;
        self.hash_key &= (1u64 << (2 * seq_len)) - 1;
    }

    fn to_str(self, seq_len: usize) -> String {
        let nucleotide = ['A', 'C', 'T', 'G'];
        let mut str = String::with_capacity(seq_len);
        for i in (0..seq_len).rev() {
            str.push(nucleotide[((self.hash_key >> (2 * i)) & 0b11) as usize]);
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
fn read_file(file_name: &str) -> Genome {
    let mut buf = BufReader::new(File::open(file_name).expect("file found"));
    let mut bytes = Vec::with_capacity(FILE_BUFFER_SIZE);
    let mut line = Vec::with_capacity(FILE_LINE_SIZE);
    let mut genome_start = false;
    while let Ok(b) = buf.read_until(b'\n', &mut line) {
        match (genome_start, b, line.starts_with(FILE_START.as_bytes())) {
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
    let mut haystack = genome.iter();
    let mut needle = Seq::default();
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

fn par_seq_cnt(genome: &Genome) -> ThrdPool {
    Iterator::collect(SEQ_STRS.into_iter().map(|seq_str| {
        let genome = Arc::clone(genome);
        spawn(move || (seq_str, seq_cnt(genome_iter(seq_str.len(), &genome))))
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

fn show_seq_pcts(g_iter: GenomeIter) -> String {
    let (seq_len, seq_cnts) = (g_iter.seq_len, seq_cnt(g_iter));
    let mut str = Vec::default();
    for (seq, pct) in calc_pcts(seq_cnts) {
        str.push(format!("{} {:.*}", seq.to_str(seq_len), DECIMALS, pct));
    }
    str.join("\n")
}

fn show_seq_cnts(thrds: ThrdPool) -> String {
    let mut str = Vec::default();
    for thrd in thrds.into_iter().rev() {
        let (seq_str, seq_cnts) = thrd.join().expect("thread halts");
        str.push(format!("{}\t{}", seq_cnts[&Seq::from(seq_str)], seq_str));
    }
    str.join("\n")
}
