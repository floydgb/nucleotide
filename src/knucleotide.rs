// The Computer Language Benchmarks Game
// https://programming-language-benchmarks.vercel.app/problem/knucleotide
//
// contributed by Greg Floyd

// Based on k-nucleotide Rust #8

// Imports --------------------------------------------------------------------
#[rustfmt::skip] 
use {
    crate::str, hashbrown::HashMap,
    std::{fs::File, io::{BufRead, BufReader}, slice, thread}
};

// Types ----------------------------------------------------------------------
#[derive(Hash, Default, PartialEq, Eq, Clone, Copy)]
struct Sequence {
    hash_key: u64,
}

struct KNucleotides<'a> {
    seq_len: usize,
    cur_seq: Sequence,
    genome_iter: slice::Iter<'a, u8>,
}

type Genome = Vec<u8>;
type ThreadPool = Vec<thread::JoinHandle<(String, u32)>>;

// Public Functions -----------------------------------------------------------
#[rustfmt::skip]
pub fn run() {
    let genome = open_file("250000_in");
    let seqs = str!["GGT","GGTA","GGTATT","GGTATTTTAATT","GGTATTTTAATTTATAGT"];

    let seqs_cnt = count_par(seqs, &genome);
    let k1_seqs_pct = count_k(1, &genome);
    let k2_seqs_pct = count_k(2, &genome);

    println!("{}\n", show_k(1, k1_seqs_pct));
    println!("{}\n", show_k(2, k2_seqs_pct));
    println!("{}", show(seqs_cnt));
}

// Traits ---------------------------------------------------------------------
impl<'a> Iterator for KNucleotides<'a> {
    type Item = Sequence;

    fn next(&mut self) -> Option<Sequence> {
        self.genome_iter.next().map(|&byte| {
            self.cur_seq.push(byte, self.seq_len);
            self.cur_seq
        })
    }
}

// Private Functions ----------------------------------------------------------
impl Sequence {
    fn push(&mut self, byte: u8, seq_len: usize) {
        self.hash_key <<= 2;
        self.hash_key |= ((byte >> 1) & 0b11) as u64;
        self.hash_key &= (1u64 << (2 * seq_len)) - 1;
    }

    fn to_str(self, seq_len: usize) -> String {
        let (mut str, nucleotides) = (String::default(), ['A', 'C', 'T', 'G']);
        for seq_i in (0..seq_len).rev() {
            let str_i = ((self.hash_key >> (2 * seq_i)) & 0b11) as usize;
            str.push(nucleotides[str_i]);
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

fn k_nucleotides(seq_len: usize, genome: &Genome) -> KNucleotides {
    KNucleotides {
        seq_len,
        cur_seq: Sequence::default(),
        genome_iter: genome.iter(),
    }
}

fn open_file(file_name: &str) -> Genome {
    match File::open(file_name) {
        Ok(file) => read_file(BufReader::new(file)),
        _ => unreachable!("file not found"),
    }
}

fn read_file(mut buf: BufReader<File>) -> Genome {
    let (mut bytes, mut line, mut start) = (Vec::new(), Vec::new(), false);
    while let Ok(bytes_read) = buf.read_until(b'\n', &mut line) {
        match bytes_read {
            0 => break,
            _ if start => bytes.extend_from_slice(&line[..line.len() - 1]),
            _ => start |= line.starts_with(">THREE".as_bytes()),
        }
        line.clear();
    }
    bytes
}

fn count_par(seq_strs: Vec<String>, genome: &Genome) -> ThreadPool {
    Iterator::collect(seq_strs.into_iter().map(|str| {
        let genome = Vec::clone(genome);
        thread::spawn(move || count(&str, str.len(), &genome))
    }))
}

fn count(seq_str: &str, seq_len: usize, genome: &Genome) -> (String, u32) {
    let (target_seq, mut seq_cnt) = (Sequence::from_str(seq_str), 0);
    for seq in k_nucleotides(seq_len, genome) {
        if seq == target_seq {
            seq_cnt += 1
        }
    }
    (seq_str.into(), seq_cnt)
}

fn count_k(seq_len: usize, genome: &Genome) -> HashMap<Sequence, u32> {
    let mut seq_cnts = HashMap::<Sequence, u32>::default();
    for seq in k_nucleotides(seq_len, genome) {
        *seq_cnts.entry(seq).or_insert(0) += 1;
    }
    seq_cnts
}

fn calc_percents(seq_cnts: HashMap<Sequence, u32>) -> Vec<(Sequence, f32)> {
    let (mut pcts, tot_seqs) = (Vec::new(), seq_cnts.values().sum::<u32>());
    for (seq, cnt) in sort_by_count(seq_cnts) {
        let percent = cnt as f32 / tot_seqs as f32 * 100_f32;
        pcts.push((seq, percent));
    }
    pcts
}

fn sort_by_count(seq_cnts: HashMap<Sequence, u32>) -> Vec<(Sequence, u32)> {
    let mut seq_cnts_sort: Vec<_> = seq_cnts.into_iter().collect();
    seq_cnts_sort.sort_by(|(_, l_cnt), (_, r_cnt)| r_cnt.cmp(l_cnt));
    seq_cnts_sort
}

fn show(pool: ThreadPool) -> String {
    let mut str = Vec::default();
    for thrd in pool.into_iter().rev() {
        let (seq_str, seq_cnt) = thrd.join().expect("thread halts");
        str.push(format!("{}\t{}", seq_cnt, seq_str));
    }
    str.join("\n")
}

fn show_k(seq_len: usize, seq_cnts: HashMap<Sequence, u32>) -> String {
    let mut str = Vec::default();
    for (seq, pct) in calc_percents(seq_cnts) {
        str.push(format!("{} {:.3}", seq.to_str(seq_len), pct));
    }
    str.join("\n")
}
