#![allow(dead_code)]
use crate::{LineReader, LinesParse, LinesParseMap, ParseControlFlow};

use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{self, BufReader, Lines, Read},
    marker::PhantomData,
    num::ParseIntError,
    ops::ControlFlow::{self, Break, Continue},
    str::FromStr,
    vec,
};

pub fn day05_file() -> io::Result<File> {
    super::input("day05")
}

#[derive(Debug, Copy, Clone)]
struct Crate {
    value: char,
}

impl Crate {
    const PREFIX: char = '[';
    const SUFFIX: char = ']';
    const LEN: usize = 3;

    const PREFIX_ERR: CrateParseError = CrateParseError::Prefix(Self::PREFIX);
    const SUFFIX_ERR: CrateParseError = CrateParseError::Suffix(Self::SUFFIX);
}

impl From<char> for Crate {
    fn from(value: char) -> Self {
        Self { value }
    }
}

impl FromStr for Crate {
    type Err = CrateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().strip_prefix(Self::PREFIX).ok_or(Self::PREFIX_ERR)?;
        let s = s.trim().strip_suffix(Self::SUFFIX).ok_or(Self::SUFFIX_ERR)?;
        let mut chars = s.chars();
        let value = chars.next().ok_or(Self::Err::Id)?;
        let None = chars.next() else { return Err(Self::Err::TooMany) };
        Ok(Self { value })
    }
}

#[derive(Debug, Default)]
struct CrateRow {
    row: Vec<Option<Crate>>,
}

impl CrateRow {
    const CHUNK_LEN: usize = Crate::LEN + 1;

    fn try_push(&mut self, chunk: impl IntoIterator<Item = char>) -> Result<(), CrateParseError> {
        let slot = String::from_iter(chunk);
        let slot = slot.trim();
        match slot.is_empty() {
            true => self.row.push(None),
            false => self.row.push(Some(Crate::from_str(slot)?)),
        }
        Ok(())
    }
}

impl IntoIterator for CrateRow {
    type Item = Option<Crate>;
    type IntoIter = CrateRowIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

#[derive(Debug)]
struct CrateRowIntoIter {
    iter: vec::IntoIter<Option<Crate>>,
}

impl CrateRowIntoIter {
    fn new(row: CrateRow) -> Self {
        let iter = row.row.into_iter();
        Self { iter }
    }
}

impl Iterator for CrateRowIntoIter {
    type Item = Option<Crate>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl FromStr for CrateRow {
    type Err = CrateParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut this = Self::default();
        let mut chunks = s.chars().array_chunks::<{ Self::CHUNK_LEN }>();
        for chunk in chunks.by_ref() {
            this.try_push(chunk)?;
        }
        if let Some(remainder) = chunks.into_remainder() {
            this.try_push(remainder)?;
        }
        Ok(this)
    }
}

#[derive(Debug)]
struct CrateRowReader<R>(LineReader<R>);

impl<R: Read> CrateRowReader<R> {
    pub fn with_position(read: R, pos: usize) -> Self {
        let reader = LineReader::with_position(read, pos);
        Self(reader)
    }

    pub fn rows(self) -> Result<CrateRows, CrateRowReaderError> {
        CrateRows::new(self)
    }
}

impl<R> ParseControlFlow for CrateRowReader<R> {
    type Item = CrateRow;
    type ParseError = CrateParseError;

    fn parse(s: &str) -> ControlFlow<Result<Self::Item, Self::ParseError>, ()> {
        // We can't trim lines before parsing, that would exclude trailing
        // and leading empty slots, which will be represented as None.
        match s.trim().is_empty() {
            true => Continue(()),
            false => Break(s.parse()),
        }
    }
}

impl<R: Read> LinesParse for CrateRowReader<R> {
    type Error = CrateRowReaderErrorSource;
    type Lines = Lines<BufReader<R>>;

    fn lines(&mut self) -> &mut Self::Lines {
        self.0.lines()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> LinesParseMap for CrateRowReader<R> {
    type Result = Result<Self::Item, CrateRowReaderError>;

    fn map(&self, res: Result<Self::Item, Self::Error>) -> Self::Result {
        res.map_err(|err| CrateRowReaderError::new(self.0.pos(), err))
    }
}

impl<R: Read> Iterator for CrateRowReader<R> {
    type Item = Result<CrateRow, CrateRowReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_map()
    }
}

#[derive(Debug)]
struct CrateRows {
    rows: Vec<CrateRow>,
}

impl CrateRows {
    fn new<R: Read>(reader: CrateRowReader<R>) -> Result<Self, CrateRowReaderError> {
        let mut rows = Vec::new();
        for row in reader {
            rows.push(row?);
        }
        Ok(Self { rows })
    }
}

impl Iterator for CrateRows {
    type Item = CrateRow;

    fn next(&mut self) -> Option<Self::Item> {
        self.rows.pop()
    }
}

#[derive(Debug, Default, Clone)]
struct Stack {
    stack: Vec<Crate>,
}

impl Stack {
    fn push(&mut self, krate: Crate) {
        self.stack.push(krate);
    }

    fn pop(&mut self) -> Option<Crate> {
        self.stack.pop()
    }
}

#[derive(Debug, Copy, Clone)]
struct Unchecked;
#[derive(Debug, Copy, Clone)]
struct Checked;

#[derive(Debug, Copy, Clone)]
struct Route<C> {
    orig: usize,
    dest: usize,
    _pd: PhantomData<C>,
}

impl<C> Route<C> {
    const PREFIX: &str = "from";
    const DELIM: &str = "to";

    const PREFIX_ERR: RouteParseError = RouteParseError::Prefix(Self::PREFIX);
    const DELIM_ERR: RouteParseError = RouteParseError::Delim(Self::DELIM);

    #[inline]
    fn orig_err(&self) -> RouteError {
        RouteError::Orig(self.orig)
    }

    #[inline]
    fn dest_err(&self) -> RouteError {
        RouteError::Dest(self.dest)
    }
}

impl Route<Unchecked> {
    fn check(self, layout: &StacksLayout) -> Result<Route<Checked>, RouteError> {
        let map = layout.map();
        let orig = *map.get(&self.orig).ok_or_else(|| self.orig_err())?;
        let dest = *map.get(&self.dest).ok_or_else(|| self.dest_err())?;
        Ok(Route::<Checked> { orig, dest, _pd: PhantomData })
    }
}

impl Route<Checked> {
    pub fn orig(&self) -> usize {
        self.orig
    }

    pub fn dest(&self) -> usize {
        self.dest
    }
}

impl FromStr for Route<Unchecked> {
    type Err = RouteParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().strip_prefix(Self::PREFIX).ok_or(Self::PREFIX_ERR)?;
        let (orig, dest) = s.split_once(Self::DELIM).ok_or(Self::DELIM_ERR)?;
        let orig = orig.trim().parse().map_err(Self::Err::orig)?;
        let dest = dest.trim().parse().map_err(Self::Err::dest)?;
        Ok(Self { orig, dest, _pd: PhantomData })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Lift {
    moves: usize,
    route: Route<Unchecked>,
}

impl Lift {
    const PREFIX: &str = "move ";
    const DELIM: char = ' ';

    const PREFIX_ERR: LiftParseError = LiftParseError::Prefix(Self::PREFIX);
    const DELIM_ERR: LiftParseError = LiftParseError::Delim(Self::DELIM);
}

impl FromStr for Lift {
    type Err = LiftParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().strip_prefix(Self::PREFIX).ok_or(Self::PREFIX_ERR)?;
        let (moves, route) = s.split_once(Self::DELIM).ok_or(Self::DELIM_ERR)?;
        let moves = moves.trim().parse().map_err(Self::Err::from)?;
        let route = route.trim().parse().map_err(Self::Err::from)?;
        Ok(Self { moves, route })
    }
}

#[derive(Debug)]
pub struct LiftReader<R>(LineReader<R>);

impl<R: Read> LiftReader<R> {
    pub fn with_position(read: R, pos: usize) -> Self {
        let reader = LineReader::with_position(read, pos);
        Self(reader)
    }
}

impl<R> ParseControlFlow for LiftReader<R> {
    type Item = Lift;
    type ParseError = LiftParseError;
}

impl<R: Read> LinesParse for LiftReader<R> {
    type Error = LiftReaderErrorSource;
    type Lines = Lines<BufReader<R>>;

    fn lines(&mut self) -> &mut Self::Lines {
        self.0.lines()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> LinesParseMap for LiftReader<R> {
    type Result = Result<Self::Item, LiftReaderError>;

    fn map(&self, res: Result<Self::Item, Self::Error>) -> Self::Result {
        res.map_err(|err| LiftReaderError::new(self.0.pos(), err))
    }
}

impl<R: Read> Iterator for LiftReader<R> {
    type Item = Result<Lift, LiftReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_map()
    }
}

#[derive(Debug)]
pub struct Platform {
    stacks: Vec<Stack>,
    layout: StacksLayout,
}

impl Platform {
    fn new(layout: StacksLayout) -> Self {
        let len = layout.len();
        let mut stacks = Vec::with_capacity(len);
        for _ in 0..len {
            stacks.push(Default::default());
        }
        Self { stacks, layout }
    }

    fn insert_row(&mut self, row: CrateRow) {
        for (pos, krate) in row.into_iter().enumerate() {
            if let Some(krate) = krate {
                self.stacks[pos].push(krate);
            }
        }
    }

    fn fill(&mut self, rows: impl IntoIterator<Item = CrateRow>) {
        for row in rows {
            self.insert_row(row)
        }
    }

    fn lift(&mut self, moves: usize, route: Route<Unchecked>) -> Result<(), RouteError> {
        let route = route.check(&self.layout)?;
        for _ in 0..moves {
            // PANIC: route was checked prior to this point, so we have access
            // to `orig` and `dest` APIs. Checked routes return bounded indexes.
            match self.stacks[route.orig()].pop() {
                Some(krate) => self.stacks[route.dest()].push(krate),
                None => break,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct StacksLayout {
    layout: HashMap<usize, usize>,
}

impl StacksLayout {
    pub fn len(&self) -> usize {
        self.layout.len()
    }

    fn map(&self) -> &HashMap<usize, usize> {
        &self.layout
    }
}

impl FromStr for StacksLayout {
    type Err = StacksLayoutParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut layout = HashMap::new();
        for (idx, id) in s.split_whitespace().into_iter().enumerate() {
            let id = id.parse()?;
            if let Entry::Vacant(entry) = layout.entry(id) {
                entry.insert(idx);
            } else {
                return Err(Self::Err::Duplicate(id));
            }
        }
        Ok(Self { layout })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error at line: {pos}, {source}")]
pub struct CrateRowReaderError {
    pos: usize,
    #[source]
    source: CrateRowReaderErrorSource,
}

impl CrateRowReaderError {
    fn new(pos: usize, source: CrateRowReaderErrorSource) -> Self {
        Self { pos, source }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CrateRowReaderErrorSource {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] CrateParseError),
}

#[derive(Debug, thiserror::Error)]
#[error("error at line: {pos}, {source}")]
pub struct LiftReaderError {
    pos: usize,
    #[source]
    source: LiftReaderErrorSource,
}

impl LiftReaderError {
    fn new(pos: usize, source: LiftReaderErrorSource) -> Self {
        Self { pos, source }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LiftReaderErrorSource {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] LiftParseError),
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum CrateParseError {
    #[error("missing left delimeter: `{0}`")]
    Prefix(char),
    #[error("missing right delimeter: `{0}`")]
    Suffix(char),
    #[error("missing crate's identifier")]
    Id,
    #[error("crate must be identified with a single char")]
    TooMany,
}

#[derive(Debug, thiserror::Error)]
pub enum RouteParseError {
    #[error("missing route prefix: `{0}`")]
    Prefix(&'static str),
    #[error("missing route delimeter: `{0}`")]
    Delim(&'static str),
    #[error(transparent)]
    Orig(#[from] OrigParseError),
    #[error(transparent)]
    Dest(#[from] DestParseError),
}

impl RouteParseError {
    fn orig(err: ParseIntError) -> Self {
        Self::Orig(OrigParseError(err))
    }

    fn dest(err: ParseIntError) -> Self {
        Self::Dest(DestParseError(err))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("could not parse route origin: {0}")]
pub struct OrigParseError(#[from] ParseIntError);

#[derive(Debug, thiserror::Error)]
#[error("could not parse route destination: {0}")]
pub struct DestParseError(#[from] ParseIntError);

#[derive(Debug, thiserror::Error)]
pub enum LiftParseError {
    #[error("missing lift prefix: `{0}`")]
    Prefix(&'static str),
    #[error("missing lift delimeter: `{0}`")]
    Delim(char),
    #[error(transparent)]
    Route(#[from] RouteParseError),
    #[error("could not parse quantity: {0}")]
    Qnt(#[from] ParseIntError),
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum RouteError {
    #[error("invalid stack origin: {0}")]
    Orig(usize),
    #[error("invalid stack destination: {0}")]
    Dest(usize),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum StacksLayoutParseError {
    #[error("duplicate stack position: {0}")]
    Duplicate(usize),
    #[error(transparent)]
    Parse(#[from] ParseIntError),
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    const STACKS: &str = "    [S] [C]         [Z]            
[F] [J] [P]         [T]     [N]    
[G] [H] [G] [Q]     [G]     [D]    
[V] [V] [D] [G] [F] [D]     [V]    
[R] [B] [F] [N] [N] [Q] [L] [S]    
[J] [M] [M] [P] [H] [V] [B] [B] [D]
[L] [P] [H] [D] [L] [F] [D] [J] [L]
[D] [T] [V] [M] [J] [N] [F] [M] [G]";

    const LAYOUT: &str = " 1   2   3   4   5   6   7   8   9 ";

    #[test]
    fn read_rows() {
        let input = Cursor::new(STACKS);
        let rows = CrateRowReader::with_position(input, 0).rows().unwrap();
        rows.for_each(|row| println!("{row:?}"));
    }

    #[test]
    fn read_layout() {
        let layout = StacksLayout::from_str(LAYOUT).unwrap();
        println!("{layout:?}");
    }

    #[test]
    fn read_platform() {
        let input = Cursor::new(STACKS);
        let rows = CrateRowReader::with_position(input, 0).rows().unwrap();
        let layout = StacksLayout::from_str(LAYOUT).unwrap();
        let mut platform = Platform::new(layout);
        platform.fill(rows);
        println!("{platform:#?}");
    }
}
