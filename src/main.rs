use riest::*;

use std::f64::consts::*;

const PHI: f64 = 1.618033988749894848204586834365638118_f64;
const NCONSTS: usize = 13;
const CONSTS: [f64; NCONSTS] = [0., 1., 2., 3., 4., 5., 6., 7., 8., 9., E, PI, PHI];
const COSTS: [Cost; NCONSTS] = [5, 7, 9, 13, 15, 17, 19, 21, 23, 25, 10, 14, 8];
const DSPLY: [&'static str; NCONSTS] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "e", "pi", "phi"];

fn main() {
    let target = 69420.;
    let thres = 1E-10;
    let len_lim = 100;
    let cost_lim = 44;
    let consts = consts();
    let avoidlist = &[];
    let mut searcher = SearcherBuilder {
        target,
        thres,
        init: &consts,
        range: -1E7..=1E7,
        avoidlist,
        max_cost: cost_lim,
        max_queue_len: len_lim,
    }
    .build();
    for p in searcher.search() {
        println!("{p} {}", diff(target, p.value().value()));
    }
    println!("{}", searcher.num_numbers());
    println!("{}", searcher.new_list().len());
    for i in 
}

fn consts() -> [(f64, Cost, Option<&'static str>); NCONSTS] {
    let mut i = 0;
    CONSTS.map(|n| {
        i += 1;
        (n, COSTS[i - 1], Some(DSPLY[i - 1]))
    })
}

fn diff(a: f64, b: f64) -> String {
    format!("{} {}", ["-", "+"][(a >= b) as usize], (a - b).abs())
}
