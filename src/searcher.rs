use crate::f64_wrapper::{Delta, F64Wrapper};
use crate::program::{Cost, Program};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
pub struct SearcherBuilder<'a, 'b> {
    pub target: f64,
    pub thres: f64,
    pub init: &'b [(f64, Cost, Option<&'a str>)],
    pub avoidlist: &'b [f64],
    pub max_queue_len: usize,
    pub max_cost: Cost,
    pub range: RangeInclusive<f64>,
}

#[derive(Clone)]
pub struct Searcher<'a> {
    step: u128,
    max_queue_len: usize,
    max_cost: Cost,
    target: f64,
    thres: f64,
    best_delta: f64,
    range: RangeInclusive<f64>,
    best: Option<Program<'a>>,
    numbers: BTreeMap<F64Wrapper<'a>, Program<'a>>,
    costs: BTreeSet<(Cost, F64Wrapper<'a>)>,
    queue: BTreeMap<Cost, Vec<Program<'a>>>,
    queue_numbers: BTreeMap<F64Wrapper<'a>, Cost>,
    curr: Vec<(Cost, F64Wrapper<'a>)>,
    avoidlist: Vec<F64Wrapper<'a>>,
}

impl<'a, 'b> SearcherBuilder<'a, 'b> {
    pub fn build(self) -> Searcher<'a> {
        let SearcherBuilder {
            target,
            thres,
            range,
            init,
            avoidlist,
            max_queue_len,
            max_cost,
        } = self;
        let avoidlist = avoidlist
            .into_iter()
            .map(|i| F64Wrapper::new(*i, thres, 0, None))
            .collect();
        let init: Vec<_> = init
            .iter()
            .map(|(n, c, s)| Program::const_program(F64Wrapper::new(*n, thres, *c, *s)))
            .collect();
        let nums: Vec<_> = init.iter().map(|p| (p.cost(), p.value())).collect();
        let costs = nums.iter().copied().collect();
        let map = init.into_iter().map(|p| (p.value(), p)).collect();
        Searcher {
            step: 0,
            thres,
            costs,
            range,
            avoidlist,
            max_cost,
            max_queue_len,
            target: target,
            best_delta: f64::INFINITY,
            best: None,
            queue: BTreeMap::new(),
            queue_numbers: BTreeMap::new(),
            curr: nums,
            numbers: map,
        }
    }
}

impl<'a> Searcher<'a> {
    pub fn search<'b>(&'b mut self) -> impl Iterator<Item = Program<'a>> + 'b {
        std::iter::from_fn(move || {
            if self.best_delta < self.thres {
                return None;
            }
            loop {
                if let Some(v) = self.expand() {
                    if v.is_some() {
                        break v;
                    }
                } else {
                    break None;
                }
            }
        })
    }

    pub fn num_numbers(&self) -> usize {
        self.numbers.len()
    }
    pub fn new_list(&self) -> &[(Cost, F64Wrapper<'a>)] {
        &self.curr
    }
    pub fn expand(&mut self) -> Option<Option<Program<'a>>> {
        let mut insert = |p: Program<'a>| {
            if p.cost() > self.max_cost || !self.range.contains(&p.value().value()) {
                return;
            }
            if let Some(v) = self.numbers.get(&p.value()) {
                if v.cost() <= p.cost() {
                    return;
                }
            }
            if let Some(&v) = self.queue_numbers.get(&p.value()) {
                if v <= p.cost() {
                    return;
                }
            }
            self.queue_numbers.insert(p.value(), p.cost());
            self.queue
                .entry(p.cost())
                .or_insert(Vec::new())
                .push(p);
        };

        for (_c, i) in self.curr.iter() {
            let p1 = self.numbers.get(&i).unwrap();
            for p in p1.combs1() {
                insert(p);
            }

            for (j, p2) in self.costs.iter() {
                if self.curr.contains(&(*j, *p2)) && *p2 > p1.value() {
                    continue;
                }
                let p2 = self.numbers.get(&p2).unwrap();
                for p in p1.combs2(p2) {
                    insert(p);
                }
            }
        }

        let (c, mut ps) = if let Some(v) = self.queue.pop_first() {
            v
        } else {
            return None;
        };

        for _ in self.max_queue_len..self.queue.len() {
            self.queue.pop_last().unwrap();
        }

        ps.retain(|p| {
            let (n, c) = (p.value(), p.cost());
            // dbg!(n, c);
            // let (mut na, mut nb) = (n, n);
            // *na.value_mut()-=0.5;
            // *nb.value_mut()+=0.5;
            // for i in self.queue_numbers.range(na..nb) { dbg!(i); }
            self.queue_numbers.remove(&n);
            if self.avoidlist.contains(&n) {
                false
            } else if let Some(v) = self.numbers.get(&n) {
                v.cost() > c
            } else {
                true
            }
        });
        self.curr.clear();
        self.curr.extend(ps.iter().map(|c| (c.cost(), c.value())));

        let (mut b, mut bd, t) = (None, self.best_delta, self.target);
        for p in ps {
            let n = p.value();
            if let Some(v) = self.numbers.get(&n) {
                self.costs.remove(&(v.cost(), n));
            }
            self.costs.insert((c, n));
            let d = (n.value() - t).abs();
            if d < bd {
                bd = d;
                b = Some(p.clone());
            }
            self.numbers.insert(n, p);
        }
        if b.is_some() {
            self.best = b.clone();
            self.best_delta = bd;
        }
        self.step += 1;
        self.max_cost += (self.step&1 == 0) as Cost;
        if self.step & 3 == 0 {
            for (n, c) in self.queue_numbers.clone().iter() {
                let n = F64Wrapper::new(n.value(), f64::MIN_POSITIVE, 0, None);
                if !self.queue.contains_key(c) {
                    self.queue_numbers.remove(&n).unwrap();
                }
            }
        }
        Some(b)
    }
}
