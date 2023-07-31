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
type SequenceCounts = HashMap<Code, u32>;

struct CodeIter<'a> {
    iter: Iter<'a, u8>,
    code: Code,
    mask: u64,
}

#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Code {
    code: u64,
}

// Constants ------------------------------------------------------------------
const FILE_NAME: &str = "250000_in";
const FILE_START: &str = ">THREE";
const OCCS: [&str; 5] = [
    "GGTATTTTAATTTATAGT",
    "GGTATTTTAATT",
    "GGTATT",
    "GGTA",
    "GGT",
];

// Methods --------------------------------------------------------------------
impl Code {
    fn push(&mut self, c: u8, mask: u64) {
        self.code <<= 2;
        self.code |= c as u64;
        self.code &= mask;
    }

    fn from_str(s: &str) -> Code {
        let mask = make_mask(s.len());
        let mut res = Code { code: 0 };
        for c in s.as_bytes() {
            res.push(encode_byte(c), mask);
        }
        res
    }

    fn to_string(self, frame: usize) -> String {
        let mut res = vec![];
        let mut code = self.code;
        for _ in 0..frame {
            let c = match code as u8 & 0b11 {
                c if c == encode_byte(&b'A') => b'A',
                c if c == encode_byte(&b'T') => b'T',
                c if c == encode_byte(&b'G') => b'G',
                c if c == encode_byte(&b'C') => b'C',
                _ => unreachable!(),
            };
            res.push(c);
            code >>= 2;
        }
        res.reverse();
        String::from_utf8(res).unwrap()
    }
}

impl<'a> CodeIter<'a> {
    fn new(input: &[u8], frame: usize) -> CodeIter {
        let mut iter = input.iter();
        let mut code = Code { code: 0 };
        let mask = make_mask(frame);
        for c in iter.by_ref().take(frame - 1) {
            code.push(*c, mask);
        }
        CodeIter { iter, code, mask }
    }
}

// Traits ---------------------------------------------------------------------
impl<'a> Iterator for CodeIter<'a> {
    type Item = Code;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&c| {
            self.code.push(c, self.mask);
            self.code
        })
    }
}

// Functions ------------------------------------------------------------------
#[inline(always)]
fn encode_byte(c: &u8) -> u8 {
    (c >> 1) & 0b11
}

fn make_mask(frame: usize) -> u64 {
    (1u64 << (2 * frame)) - 1
}

fn read_file() -> Vec<u8> {
    if let Ok(file) = File::open(FILE_NAME) {
        let mut reader = BufReader::new(file);
        let mut result = Vec::with_capacity(65536);
        let mut line = String::with_capacity(64);

        loop {
            match reader.read_line(&mut line) {
                Ok(b) if b > 0 => {
                    if line.starts_with(FILE_START) {
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
                    result.extend(bytes[..bytes.len() - 1].iter().map(encode_byte))
                }
                _ => break,
            }
        }
        result
    } else {
        unreachable!("File not found");
    }
}

fn gen_freq(input: &[u8], frame: usize) -> SequenceCounts {
    let mut freq = SequenceCounts::default();
    for code in CodeIter::new(input, frame) {
        *freq.entry(code).or_insert(0) += 1;
    }
    freq
}

fn print_percents(n: usize, hmap: &SequenceCounts) {
    let total = hmap.values().sum::<u32>() as f32;
    let mut v: Vec<_> = hmap.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for &(code, count) in v.iter() {
        let freq_percent = *count as f32 * 100.0 / total;
        println!("{} {:.3}", code.to_string(n), freq_percent);
    }
    println!();
}

fn print_counts(s: &str, hmap: &SequenceCounts) {
    let code = Code::from_str(s);
    if hmap.contains_key(&code) {
        println!("{}\t{}", hmap[&code], s);
    } else {
        println!("{}\t{}", 0, s);
    };
}

pub fn run() {
    let genome = Arc::new(read_file());

    // In reverse to spawn big tasks first
    let threads: Vec<_> = OCCS
        .into_iter()
        .map(|occ| {
            let input = genome.clone();
            thread::spawn(move || (occ, gen_freq(&input, occ.len())))
        })
        .collect();

    print_percents(1, &gen_freq(&genome, 1));
    print_percents(2, &gen_freq(&genome, 2));

    for t in threads.into_iter().rev() {
        if let Ok((occ, freq)) = t.join() {
            print_counts(occ, &freq);
        }
    }
}
