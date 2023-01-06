use itertools::Itertools;

use std::{
    cmp,
    fs::File,
    io::{self, Read},
    num::ParseIntError,
    ops::ControlFlow::{self, Break, Continue},
};

use crate::LineReader;

pub type Calories = u64;

pub fn day01_file() -> io::Result<File> {
    super::input("day01")
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elf {
    cals: Calories,
    pos: usize,
}

impl Elf {
    pub fn new(pos: usize, cals: Calories) -> Self {
        Self { pos, cals }
    }

    pub fn cals(&self) -> Calories {
        self.cals
    }
}

#[derive(Debug)]
pub struct ElvesReader<R>(LineReader<R>);

impl<R: Read> ElvesReader<R> {
    pub fn new(read: R) -> Self {
        let reader = LineReader::new(read);
        Self(reader)
    }

    pub fn max_by_cal(self) -> Result<Elf, ElfError> {
        let (mut max, mut elfs) = self.split_first()?;
        max = elfs.try_fold(max, |m, e| Ok::<_, ElfError>(cmp::max(m, e?)))?;
        Ok(max)
    }

    fn split_first(mut self) -> Result<(Elf, <Self as IntoIterator>::IntoIter), ElfError> {
        let max = match self.next() {
            Some(elf) => elf?,
            None => return Err(ElfError::NoEntries),
        };
        Ok((max, self))
    }

    pub fn sum_top(self, n: usize) -> Result<Calories, ElfError> {
        let ordered = self
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&b.cals(), &a.cals()));
        let sum: Calories = ordered.take(n).map(|e| e.cals()).sum();
        Ok(sum)
    }
}

impl<R: Read> Iterator for ElvesReader<R> {
    type Item = Result<Elf, ElfError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut parser = CaloriesParser::default();
        loop {
            let err = match self.0.lines.next()? {
                Ok(ref line) => match parser.parse(line) {
                    Continue(_) => continue,
                    Break(Ok(_)) => break,
                    Break(Err(err)) => Err(err.into()),
                },
                Err(err) => Err(err.into()),
            };
            return Some(err);
        }
        self.0.pos += 1;
        Some(Ok(Elf::new(self.0.pos(), parser.cals())))
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct CaloriesParser {
    filled: bool,
    cals: Calories,
}

impl CaloriesParser {
    fn parse(&mut self, s: &str) -> ControlFlow<Result<(), ParseIntError>, ()> {
        let s = s.trim();
        let (prev, curr) = (self.filled, !s.is_empty());
        self.filled = curr;
        match (prev, curr) {
            (true, true) | (false, true) => match self.add_cals(s) {
                Ok(_) => Continue(()),
                Err(err) => Break(Err(err)),
            },
            (false, false) => Continue(()),
            (true, false) => Break(Ok(())),
        }
    }

    fn add_cals(&mut self, s: &str) -> Result<(), ParseIntError> {
        let cals = s.parse::<Calories>()?;
        self.cals += cals;
        Ok(())
    }

    fn cals(&self) -> Calories {
        self.cals
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ElfError {
    #[error(transparent)]
    Parse(#[from] ParseIntError),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("no calories listed in the file")]
    NoEntries,
}
