use crate::f64_wrapper::F64Wrapper;
use std::fmt;
use Instr as I;
use Instr::*;

pub type Cost = u128;
type Is = Instr<'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instr<'a> {
    Push(F64Wrapper<'a>),
    Neg,
    Add,
    Multiply,
    Divide,
    Power,
    Logarithm,
}

impl<'a> Instr<'a> {
    pub fn cost(&self) -> Cost {
        match self {
            I::Push(v) => v.cost(),
            I::Neg => 5,
            I::Add => 6,
            I::Multiply => 15,
            I::Divide => 18,
            I::Power => 15,
            I::Logarithm => 19,
        }
    }

    pub fn display(&self) -> String {
        String::from(match self {
            I::Push(v) => return v.to_string(),
            I::Neg => "-",
            I::Add => "+",
            I::Multiply => "*",
            I::Divide => "/",
            I::Power => "^",
            I::Logarithm => "log",
        })
    }

    pub fn arity(&self) -> u8 {
        match self {
            I::Push(_) => 0,
            I::Neg => 1,
            I::Add | I::Multiply | I::Divide | I::Power | I::Logarithm => 2,
        }
    }

    pub fn execute0(&self) -> F64Wrapper<'a> {
        if let Self::Push(v) = self {
            *v
        } else {
            panic!("unknown nullary instruction")
        }
    }

    pub fn execute1_checked(&self, x: F64Wrapper) -> Option<F64Wrapper<'static>> {
        let thres = x.thres();
        let cost = x.cost() + self.cost();
        (match self {
            Self::Neg => Some(-x.value()),
            _ => panic!("unknown unary instruction"),
        })
        .map(|i| F64Wrapper::new(i, thres, cost, None))
    }

    pub fn execute2_checked(&self, a: F64Wrapper, b: F64Wrapper) -> Option<F64Wrapper<'static>> {
        let thres = a.thres();
        let cost = a.cost() + b.cost() + self.cost();
        let (a, b) = (a.value(), b.value());
        (match self {
            I::Add => Some(a + b),
            I::Multiply => Some(a * b),
            I::Divide => {
                if b.abs() > thres {
                    Some(a / b)
                } else {
                    None
                }
            }
            I::Power => {
                if a <= 0. {
                    None
                } else {
                    Some(a.powf(b))
                }
            }
            I::Logarithm => {
                if a >= 0. && b >= 0. {
                    Some(a.log(b))
                } else {
                    None
                }
            }
            _ => panic!("unknown binary instruction"),
        })
        .map(|i| F64Wrapper::new(i, thres, cost, None))
    }

    pub const INST1: &[Is] = &[Neg];
    // chosen when a < b (commutative)
    pub const INST_C: &[Is] = &[Add, Multiply];
    // chosen in all cases (noncommutative)
    pub const INST_NC: &[Is] = &[Divide, Power, Logarithm];
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Program<'a> {
    program: Vec<Instr<'a>>,
    cost: Cost,
    value: F64Wrapper<'a>,
}

fn c<'a, 'b, T: Iterator<Item = Program<'a>> + 'b>(
    it: T,
    arr: &'b [Instr<'a>],
    a: &'b Program<'a>,
    b: &'b Program<'a>,
) -> impl Iterator<Item = Program<'a>> + 'b {
    it.chain(arr.iter().filter_map(|i| a.comb2(b, *i)))
}

impl<'a> Program<'a> {
    pub fn new(program: Vec<Instr<'a>>) -> Self {
        let (cost, value) = Self::cv_pair(&program);
        Self {
            program,
            cost,
            value,
        }
    }

    pub fn const_program(v: F64Wrapper<'a>) -> Self {
        let i = vec![Instr::Push(v)];
        Self::new(i)
    }

    pub fn cost(&self) -> Cost {
        self.cost
    }
    pub fn value(&self) -> F64Wrapper<'a> {
        self.value
    }

    pub fn combs1<'b>(&'b self) -> impl Iterator<Item = Program<'a>> + 'b {
        Instr::INST1.iter().filter_map(|i| {
            i.execute1_checked(self.value).map(|v| {
                let mut s = self.clone();
                s.program.push(*i);
                s.cost += i.cost();
                s.value = v;
                s
            })
        })
    }

    pub fn combs2<'b>(&'b self, other: &'b Self) -> impl Iterator<Item = Program<'a>> + 'b {
        use Instr::*;

        let (mut a, mut b) = (self, other);
        if a.value > b.value {
            (a, b) = (b, a);
        }

        let ar = if a.value != b.value {
            Instr::INST_NC
        } else {
            &[]
        };

        c(
            c(c([].into_iter(), Instr::INST_C, a, b), Instr::INST_NC, a, b),
            ar,
            b,
            a,
        )
    }

    pub fn comb2(&self, other: &Self, instr: Instr<'a>) -> Option<Self> {
        assert_eq!(instr.arity(), 2);
        instr.execute2_checked(self.value, other.value).map(|v| {
            let mut s = self.clone();
            s.program.extend_from_slice(&other.program);
            s.program.push(instr);
            s.value = v;
            s.cost += other.cost + instr.cost();
            s
        })
    }

    fn cv_pair(p: &[Instr<'a>]) -> (Cost, F64Wrapper<'a>) {
        let mut cost = 0;
        let mut stack = Vec::new();
        for i in p {
            cost += i.cost();
            let arity = i.arity();
            match arity {
                0 => stack.push(i.execute0()),
                1 => {
                    let a = stack.pop().unwrap();
                    stack.push(
                        i.execute1_checked(a)
                            .expect("value should be in the domain of the function"),
                    );
                }
                2 => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(
                        i.execute2_checked(a, b)
                            .expect("value should be in the domain of the function"),
                    );
                }
                x => panic!("unknown arity for instruction: {x}"),
            }
        }
        assert_eq!(stack.len(), 1);
        (cost, stack.pop().unwrap())
    }
}

impl<'a> fmt::Display for Program<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut stack = Vec::new();
        for i in &self.program {
            let id = i.display();
            match i.arity() {
                0 => stack.push(id),
                1 => {
                    let a = stack.pop().unwrap();
                    stack.push(format!("({}{})", id, a));
                }
                2 => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(format!("({} {} {})", a, id, b));
                }
                x => panic!("unknown arity for instruction: {x}"),
            }
        }
        assert_eq!(stack.len(), 1);
        write!(f, "{}", stack.pop().unwrap())
    }
}
