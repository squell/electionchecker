use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

pub type Count = u64;

#[derive(Clone, Debug, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Votes(pub Count);

#[derive(Clone, Debug, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Seats {
    awarded: Count,
}

impl Seats {
    pub fn transfer(&mut self, pool: &mut Seats) {
        self.awarded += 1;
        pool.awarded -= 1;
    }

    pub fn count(&self) -> Count {
        self.awarded
    }

    pub fn filled(awarded: Count) -> Self {
        Seats { awarded }
    }

    pub fn unlimited() -> Self {
        Self::filled(0)
    }
}

impl std::fmt::Display for Seats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.awarded)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: u64,
}

impl Ord for Fraction {
    fn cmp(&self, other: &Fraction) -> Ordering {
        (self.numerator * other.denominator).cmp(&(other.numerator * self.denominator))
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Fraction {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal)
    }
}

impl Eq for Fraction {}

impl std::fmt::Display for Fraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.denominator {
            1 => write!(f, "{}", self.numerator),
            _ => write!(f, "{}/{}", self.numerator, self.denominator),
        }
    }
}

pub fn ballotted<T>(mut vec: Vec<T>, limit: Count) -> Vec<T> {
    use rand::rng;
    use rand::seq::SliceRandom;

    let limit: usize = limit.try_into().unwrap();
    if limit < vec.len() {
        eprintln!("non-deterministic choice!");
    }

    vec.shuffle(&mut rng());
    vec.truncate(limit);

    vec
}
