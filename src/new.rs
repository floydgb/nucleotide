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
        sync::Arc,
        thread,
    },
};

// Types ----------------------------------------------------------------------
#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
struct Code {
    code: u64,
}

struct Iter<'a> {
    iter: std::slice::Iter<'a, u8>,
    code: Code,
    mask: u64,
}

type Occurance = &'static str;

// Constants ------------------------------------------------------------------
const A: u8 = (b'A' >> 1) & 0b11;
const T: u8 = (b'T' >> 1) & 0b11;
const G: u8 = (b'G' >> 1) & 0b11;
const C: u8 = (b'C' >> 1) & 0b11;

// Functions ------------------------------------------------------------------
impl Code {
    #[inline(always)]
    fn push(&mut self, c: u8, mask: u64) {
        self.code <<= 2;
        self.code |= c as u64;
        self.code &= mask;
    }

    #[inline(always)]
    fn to_string(self, frame: usize) -> String {
        let mut res = vec![];
        let mut code = self.code;
        for _ in 0..frame {
            let c = match code as u8 & 0b11 {
                A => b'A',
                T => b'T',
                G => b'G',
                C => b'C',
                _ => unreachable!(),
            };
            res.push(c);
            code >>= 2;
        }
        res.reverse();
        String::from_utf8(res).unwrap()
    }
}

impl<'a> Iter<'a> {
    #[inline(always)]
    fn new(input: &[u8], frame: usize) -> Iter {
        let mut iter = input.iter();
        let mut code = Code { code: 0 };
        let mask = make_mask(frame);
        for c in iter.by_ref().take(frame - 1) {
            code.push(*c, mask);
        }
        Iter { iter, code, mask }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Code;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&c| {
            self.code.push(c, self.mask);
            self.code
        })
    }
}

#[inline(always)]
fn from_str(s: &str) -> Code {
    let mut res = Code { code: 0 };
    for c in s.as_bytes() {
        res.push(encode_byte(c), make_mask(s.len()));
    }
    res
}

#[inline(always)]
fn make_mask(frame: usize) -> u64 {
    (1u64 << (2 * frame)) - 1
}

#[inline(always)]
fn encode_byte(c: &u8) -> u8 {
    (c >> 1) & 0b11
}

#[inline(always)]
fn gen_freq(input: &[u8], frame: usize) -> HashMap<Code, u32> {
    let mut freq = HashMap::<Code, u32>::default();
    for code in Iter::new(input, frame) {
        *freq.entry(code).or_insert(0) += 1;
    }
    freq
}

#[inline(always)]
fn print_freq(frame: usize, freqs: &HashMap<Code, u32>) {
    let mut v: Vec<_> = freqs.iter().map(|(&code, &freq)| (freq, code)).collect();
    v.sort();
    let total_freqs = v.iter().map(|&(freq, _)| freq).sum::<u32>() as f32;
    for &(freq, key) in v.iter().rev() {
        println!(
            "{} {:.3}",
            key.to_string(frame),
            (freq as f32 * 100.) / total_freqs
        );
    }
    println!();
}

fn print_freqs(occ: &Occurance, freqs: &HashMap<Code, u32>) {
    let count = if freqs.contains_key(&from_str(occ)) {
        freqs[&from_str(occ)]
    } else {
        0
    };
    println!("{}\t{}", count, occ);
}

#[inline(always)]
fn read_input() -> Vec<u8> {
    // let file_name = std::env::args_os()
    //     .nth(1)
    //     .and_then(|s| s.into_string().ok())
    //     .unwrap_or("250000_in".into());
    let mut r = BufReader::new(File::open("250000_in").unwrap());
    let mut res = Vec::with_capacity(65536);
    let mut line = String::with_capacity(64);
    loop {
        match r.read_line(&mut line) {
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
        match r.read_line(&mut line) {
            Ok(b) if b > 0 => {
                let bytes = line.as_bytes();
                res.extend(bytes[..bytes.len() - 1].iter().map(encode_byte))
            }
            _ => break,
        }
    }
    res
}

pub fn run() {
    let occs = vec![
        "GGTATTTTAATTTATAGT",
        "GGTATTTTAATT",
        "GGTATT",
        "GGTA",
        "GGT",
    ];
    let input = Arc::new(read_input());

    // In reverse to spawn big tasks first
    let results: Vec<_> = occs
        .into_iter()
        .map(|item| {
            let input = input.clone();
            thread::spawn(move || (item, gen_freq(&input, item.len())))
        })
        .collect();

    print_freq(1, &gen_freq(&input, 1));
    print_freq(2, &gen_freq(&input, 2));

    for t in results.into_iter().rev() {
        let (occ, freq) = t.join().unwrap();
        print_freqs(&occ, &freq);
    }
}
