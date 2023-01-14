use crate::{LineReader, LinesParse, LinesParseMap, ParseControlFlow};

use std::{
    fs::File,
    io::{self, BufReader, Lines, Read},
    num::ParseIntError,
    ops::RangeInclusive,
    str::FromStr,
};

pub fn day04_file() -> io::Result<File> {
    crate::input("day04")
}

type SectionId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sections {
    range: RangeInclusive<SectionId>,
}

impl Sections {
    const DELIMETER: char = '-';

    pub fn contains(&self, other: &Self) -> bool {
        let start = self.range.start() <= other.range.start();
        let end = self.range.end() >= other.range.end();
        start && end
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        let start = self.range.start() <= other.range.end();
        let end = self.range.end() >= other.range.start();
        start && end
    }
}

impl FromStr for Sections {
    type Err = SectionsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bounds = s.trim().split(Self::DELIMETER);
        let start = bounds.next().ok_or(Self::Err::START)?.trim().parse()?;
        let end = bounds.next().ok_or(Self::Err::END)?.trim().parse()?;
        let None = bounds.next() else { return Err(Self::Err::Trailing) };
        Ok(Self { range: RangeInclusive::new(start, end) })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionsPair(Sections, Sections);

impl SectionsPair {
    const DELIMETER: char = ',';

    pub fn contains(&self) -> bool {
        self.0.contains(&self.1) || self.1.contains(&self.0)
    }

    pub fn overlaps(&self) -> bool {
        self.0.overlaps(&self.1) || self.1.overlaps(&self.0)
    }
}

impl FromStr for SectionsPair {
    type Err = SectionsPairError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sections = s.trim().split(Self::DELIMETER);
        let sec0 = sections.next().ok_or(Self::Err::FIRST)?.trim().parse()?;
        let sec1 = sections.next().ok_or(Self::Err::SECOND)?.trim().parse()?;
        let None = sections.next() else { return Err(Self::Err::Trailing) };
        Ok(Self(sec0, sec1))
    }
}

#[derive(Debug)]
pub struct SectionsPairReader<R>(LineReader<R>);

impl<R: Read> SectionsPairReader<R> {
    pub fn new(read: R) -> Self {
        let reader = LineReader::new(read);
        Self(reader)
    }

    pub fn contained_pairs(self) -> Result<usize, PairReadError> {
        self.into_iter().try_fold(0, |mut count, pair| {
            pair?.contains().then(|| count += 1);
            Ok(count)
        })
    }

    pub fn overlaped_pairs(self) -> Result<usize, PairReadError> {
        self.into_iter().try_fold(0, |mut count, pair| {
            pair?.overlaps().then(|| count += 1);
            Ok(count)
        })
    }
}

impl<R: Read> Iterator for SectionsPairReader<R> {
    type Item = Result<SectionsPair, PairReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_map()
    }
}

impl<R> ParseControlFlow for SectionsPairReader<R> {
    type Item = SectionsPair;
    type ParseError = SectionsPairError;
}

impl<R: Read> LinesParse for SectionsPairReader<R> {
    type Error = PairReadErrorSource;
    type Lines<'s> = &'s mut Lines<BufReader<R>> where Self: 's;

    fn lines(&mut self) -> Self::Lines<'_> {
        self.0.lines()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> LinesParseMap for SectionsPairReader<R> {
    type Result = Result<Self::Item, PairReadError>;

    fn map(&self, res: Result<Self::Item, Self::Error>) -> Self::Result {
        res.map_err(|err| PairReadError::new(self.0.pos(), err))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error at line: {pos}, {source}")]
pub struct PairReadError {
    pos: usize,
    #[source]
    source: PairReadErrorSource,
}

impl PairReadError {
    fn new(pos: usize, source: PairReadErrorSource) -> Self {
        Self { pos, source }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PairReadErrorSource {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] SectionsPairError),
}

#[derive(Debug, thiserror::Error)]
pub enum SectionsPairError {
    #[error(transparent)]
    Section(#[from] SectionsError),
    #[error("unexpected trailing sections' pair")]
    Trailing,
    #[error(transparent)]
    Missing(MissingPair),
}

impl SectionsPairError {
    const FIRST: Self = Self::Missing(MissingPair::First);
    const SECOND: Self = Self::Missing(MissingPair::Second);
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum MissingPair {
    #[error("missing section's pair first range")]
    First,
    #[error("missing section's pair second range")]
    Second,
}

#[derive(Debug, thiserror::Error)]
pub enum SectionsError {
    #[error(transparent)]
    Parse(#[from] ParseIntError),
    #[error("unexpected trailing sections' range")]
    Trailing,
    #[error(transparent)]
    Missing(MissingSections),
}

impl SectionsError {
    const START: Self = Self::Missing(MissingSections::Start);
    const END: Self = Self::Missing(MissingSections::End);
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum MissingSections {
    #[error("missing section's range start")]
    Start,
    #[error("missing section's range end")]
    End,
}
