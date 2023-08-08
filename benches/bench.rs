#![feature(test)]
extern crate test;

use {
    nucleotide::{knucleotide, prev},
    test::Bencher,
};

#[cfg(test)]
mod bench {
    use super::*;

    #[bench]
    fn bench_new(b: &mut Bencher) {
        b.iter(|| knucleotide::run());
    }

    #[bench]
    fn bench_prev(b: &mut Bencher) {
        b.iter(|| prev::run());
    }
}
