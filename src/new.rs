// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// contributed by Tom Kaitchuck

// Based on k-nucleotide Rust #7
// Switched to used Hashbrown and removed custom hash code.
// Removed rayon and use threads directly
// Copied the read_input function from k-nucleotide Rust #4

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
#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Code {
    code: u64,
}

struct CodeIter<'a> {
    iter: Iter<'a, u8>,
    code: Code,
    mask: u64,
}

// Constants ------------------------------------------------------------------
const SEQUENCES: [&str; 5] = [
    "GGTATTTTAATTTATAGT",
    "GGTATTTTAATT",
    "GGTATT",
    "GGTA",
    "GGT",
];

// Functions ------------------------------------------------------------------
impl Code {
    #[inline(always)]
    fn push(&mut self, c: u64, mask: u64) {
        self.code <<= 2;
        self.code |= c;
        self.code &= mask;
    }
}

impl<'a> CodeIter<'a> {
    fn new(input: &[u8], frame: usize) -> CodeIter {
        let mut iter = input.iter();
        let mut code = Code { code: 0 };
        let mask = make_mask(&frame);
        for &c in iter.by_ref().take(frame - 1) {
            code.push(c.into(), mask);
        }
        CodeIter { iter, code, mask }
    }
}

impl<'a> Iterator for CodeIter<'a> {
    type Item = Code;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&c| {
            self.code.push(c.into(), self.mask);
            self.code
        })
    }
}

#[inline(always)]
fn encode_byte(c: &u8) -> u8 {
    (c >> 1) & 0b11
}

#[inline(always)]
fn make_mask(sequence_len: &usize) -> u64 {
    (1u64 << (2 * sequence_len)) - 1
}

fn from_str(sequence: &str) -> Code {
    let mut code = Code { code: 0 };
    for c in sequence.as_bytes() {
        code.push(encode_byte(c).into(), make_mask(&sequence.len()));
    }
    code
}

fn gen_freq(input: &[u8], frame: usize) -> HashMap<Code, u32> {
    let mut freq = HashMap::<Code, u32>::default();
    for code in CodeIter::new(input, frame) {
        *freq.entry(code).or_insert(0) += 1;
    }
    freq
}

fn read_file() -> Vec<u8> {
    // let file_name = std::env::args_os()
    //     .nth(1)
    //     .and_then(|s| s.into_string().ok())
    //     .unwrap_or("250000_in".into());
    let mut reader = BufReader::new(File::open("250000_in").unwrap());
    let mut res = Vec::with_capacity(65536);
    let mut line = String::with_capacity(64);
    loop {
        match reader.read_line(&mut line) {
            Ok(b) if b > 0 => {
                if line.starts_with(">THREE") {
                    break;
                }
            }
            _ => break,
        }
        line.clear();
    }
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(b) if b > 0 => {
                let bytes = line.as_bytes();
                res.extend(bytes[..bytes.len() - 1].iter().map(encode_byte))
            }
            _ => break,
        }
    }
    res
}

fn print_freq(sequence: &str, freqs: &HashMap<Code, u32>) {
    let mut v: Vec<_> = freqs.iter().map(|(&code, &freq)| (freq, code)).collect();
    v.sort();
    let total_freqs = v.iter().map(|&(freq, _)| freq).sum::<u32>() as f32;
    for &(freq, _) in v.iter().rev() {
        let freq_percent = freq as f32 / total_freqs * 100.0;
        println!("{} {:.3}", sequence, freq_percent);
    }
    println!();
}

fn print_freqs(sequence: &str, freqs: &HashMap<Code, u32>) {
    let code = from_str(sequence);
    let freq = match freqs.contains_key(&code) {
        true => freqs[&code],
        _ => 0,
    };
    println!("{}\t{}", freq, sequence);
}

pub fn run() {
    let file_input = Arc::new(read_file());

    let results: Vec<_> = SEQUENCES
        .iter()
        .map(|item| {
            let input = file_input.clone();
            thread::spawn(move || (item, gen_freq(&input, item.len())))
        })
        .collect();

    print_freq(SEQUENCES[4], &gen_freq(&file_input, 1));
    print_freq(SEQUENCES[3], &gen_freq(&file_input, 2));
    for t in results.into_iter().rev() {
        let (next_sequence, freq) = t.join().unwrap();
        print_freqs(&next_sequence, &freq);
    }
}
