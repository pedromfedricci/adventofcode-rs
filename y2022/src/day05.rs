use crate::{LinePeeker, LinesParseIfOk, ParseControlFlow, PeekableLines};

use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{self, BufReader, Lines, Read},
    iter::{self, Peekable},
    marker::PhantomData,
    num::ParseIntError,
    ops::ControlFlow::{self, Break, Continue},
    str::FromStr,
    vec,
};

pub fn day05_file() -> io::Result<File> {
    super::input("day05")
}

pub fn drawing<R: Read>(input: R) -> (Platform, Lifts) {
    let (platform, lines) = Platform::read(input);
    let (lifts, _) = LiftPeeker::from(lines).lifts();
    (platform, lifts)
}

#[derive(Debug, Copy, Clone)]
pub struct Crate {
    value: char,
}

impl Crate {
    const PREFIX: char = '[';
    const SUFFIX: char = ']';
    const LEN: usize = 3;

    const PREFIX_ERR: CrateParseError = CrateParseError::Prefix(Self::PREFIX);
    const SUFFIX_ERR: CrateParseError = CrateParseError::Suffix(Self::SUFFIX);

    pub fn into_inner(self) -> char {
        self.value
    }
}

impl From<char> for Crate {
    fn from(value: char) -> Self {
        Self { value }
    }
}

impl From<&Crate> for char {
    fn from(krate: &Crate) -> Self {
        krate.into_inner()
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

    fn try_push<I>(&mut self, chunk: I) -> Result<(), CrateParseError>
    where
        I: IntoIterator<Item = char>,
    {
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
        chunks.by_ref().try_for_each(|chunk| this.try_push(chunk))?;
        chunks.into_remainder().map_or(Ok(()), |rem| this.try_push(rem))?;
        Ok(this)
    }
}

#[derive(Debug)]
struct CrateRowPeeker<R: Read>(LinePeeker<R>);

impl<R: Read> CrateRowPeeker<R> {
    pub fn into_inner(self) -> LinePeeker<R> {
        self.0
    }

    pub fn rows(self) -> (CrateRows, LinePeeker<R>) {
        CrateRows::new(self)
    }
}

impl<R: Read> From<LinePeeker<R>> for CrateRowPeeker<R> {
    fn from(peeker: LinePeeker<R>) -> Self {
        Self(peeker)
    }
}

impl<R: Read> ParseControlFlow for CrateRowPeeker<R> {
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

impl<R: Read> LinesParseIfOk for CrateRowPeeker<R> {
    type InnerIter = Lines<BufReader<R>>;
    type Peekable<'s> = &'s mut Peekable<Self::InnerIter> where Self: 's;

    fn peekable(&mut self) -> Self::Peekable<'_> {
        self.0.peekable()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> Iterator for CrateRowPeeker<R> {
    type Item = CrateRow;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_if_ok()
    }
}

#[derive(Debug, Default)]
struct CrateRows {
    rows: Vec<CrateRow>,
}

impl CrateRows {
    fn new<R: Read>(mut peeker: CrateRowPeeker<R>) -> (CrateRows, LinePeeker<R>) {
        let rows = (&mut peeker).collect();
        let peeker = peeker.into_inner();
        (Self { rows }, peeker)
    }
}

impl Iterator for CrateRows {
    type Item = CrateRow;

    // Must iterate from the bottom of the platform up to the top.
    fn next(&mut self) -> Option<Self::Item> {
        self.rows.pop()
    }
}

#[derive(Debug, Default, Clone)]
struct Stack {
    stack: Vec<Crate>,
}

impl Stack {
    pub fn push(&mut self, krate: Crate) {
        self.stack.push(krate);
    }

    pub fn last(&self) -> Option<&'_ Crate> {
        self.stack.last()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn lift(&mut self, start: usize) -> StackLift<'_> {
        let drain = self.stack.drain(start..);
        StackLift { drain }
    }

    pub fn lift_rev(&mut self, start: usize) -> StackLiftRev<'_> {
        let rev = self.lift(start).rev();
        StackLiftRev { rev }
    }
}

impl Extend<Crate> for Stack {
    fn extend<T: IntoIterator<Item = Crate>>(&mut self, iter: T) {
        self.stack.extend(iter)
    }
}

#[derive(Debug)]
struct StackLift<'a> {
    drain: vec::Drain<'a, Crate>,
}

impl Iterator for StackLift<'_> {
    type Item = Crate;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }
}

impl DoubleEndedIterator for StackLift<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

#[derive(Debug)]
struct StackLiftRev<'a> {
    rev: iter::Rev<StackLift<'a>>,
}

impl<'a> Iterator for StackLiftRev<'a> {
    type Item = Crate;

    fn next(&mut self) -> Option<Self::Item> {
        self.rev.next()
    }
}

impl DoubleEndedIterator for StackLiftRev<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.rev.next_back()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Unchecked;
#[derive(Debug, Copy, Clone)]
pub struct Checked;

#[derive(Copy, Clone)]
pub struct Route<C> {
    orig: usize,
    dest: usize,
    _pd: PhantomData<C>,
}

type RouteUnchecked = Route<Unchecked>;
type RouteChecked = Route<Checked>;

impl<C: Debug> Debug for Route<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use std::any::type_name;
        f.debug_struct("Route")
            .field("orig", &self.orig)
            .field("dest", &self.dest)
            .field("C", &type_name::<C>())
            .finish()
    }
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

impl RouteUnchecked {
    fn check(self, layout: &Layout) -> Result<RouteChecked, RouteError> {
        let map = layout.map();
        let orig = *map.get(&self.orig).ok_or_else(|| self.orig_err())?;
        let dest = *map.get(&self.dest).ok_or_else(|| self.dest_err())?;
        Ok(Route::<Checked> { orig, dest, _pd: PhantomData })
    }
}

impl RouteChecked {
    pub fn orig(&self) -> usize {
        self.orig
    }

    pub fn dest(&self) -> usize {
        self.dest
    }
}

impl FromStr for RouteUnchecked {
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
pub struct Lift<C> {
    moves: usize,
    route: Route<C>,
}

type LiftUnchecked = Lift<Unchecked>;
type LiftChecked = Lift<Checked>;

impl<C> Lift<C> {
    const PREFIX: &str = "move ";
    const DELIM: char = ' ';

    const PREFIX_ERR: LiftParseError = LiftParseError::Prefix(Self::PREFIX);
    const DELIM_ERR: LiftParseError = LiftParseError::Delim(Self::DELIM);

    pub fn moves(&self) -> usize {
        self.moves
    }

    pub fn route(&self) -> &Route<C> {
        &self.route
    }
}

impl LiftUnchecked {
    fn check(self, layout: &Layout) -> Result<LiftChecked, RouteError> {
        let route = self.route.check(layout)?;
        let moves = self.moves;
        Ok(Lift::<Checked> { route, moves })
    }
}

impl FromStr for LiftUnchecked {
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
pub struct LiftPeeker<R: Read>(LinePeeker<R>);

impl<R: Read> LiftPeeker<R> {
    fn into_inner(self) -> LinePeeker<R> {
        self.0
    }

    fn lifts(self) -> (Lifts, LinePeeker<R>) {
        Lifts::new(self)
    }
}

impl<R: Read> From<LinePeeker<R>> for LiftPeeker<R> {
    fn from(peeker: LinePeeker<R>) -> Self {
        Self(peeker)
    }
}

#[derive(Debug)]
pub struct Lifts {
    lifts: vec::IntoIter<LiftUnchecked>,
}

impl Lifts {
    fn new<R: Read>(mut peeker: LiftPeeker<R>) -> (Self, LinePeeker<R>) {
        let lifts = (&mut peeker).collect::<Vec<_>>().into_iter();
        let peeker = peeker.into_inner();
        (Self { lifts }, peeker)
    }
}

impl Iterator for Lifts {
    type Item = LiftUnchecked;

    fn next(&mut self) -> Option<Self::Item> {
        self.lifts.next()
    }
}

impl<R: Read> ParseControlFlow for LiftPeeker<R> {
    type Item = LiftUnchecked;
    type ParseError = LiftParseError;
}

impl<R: Read> LinesParseIfOk for LiftPeeker<R> {
    type InnerIter = Lines<BufReader<R>>;
    type Peekable<'s> = &'s mut PeekableLines<R> where Self: 's;

    fn peekable(&mut self) -> Self::Peekable<'_> {
        self.0.peekable()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> Iterator for LiftPeeker<R> {
    type Item = LiftUnchecked;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_if_ok()
    }
}

#[derive(Debug)]
struct StackPairMut<'a> {
    pub orig: &'a mut Stack,
    pub dest: &'a mut Stack,
}

struct LiftParams<'a> {
    pub pair: StackPairMut<'a>,
    pub pos: usize,
}

#[derive(Debug)]
pub struct Platform {
    stacks: Vec<Stack>,
    layout: Layout,
}

impl Platform {
    pub fn read<R: Read>(read: R) -> (Self, LinePeeker<R>) {
        let peeker = LinePeeker::new(read);
        let (rows, peeker) = CrateRowPeeker::from(peeker).rows();
        let (layout, peeker) = LayoutPeeker::from(peeker).layout();
        let mut platform = Platform::new(layout);
        platform.extend(rows);
        (platform, peeker)
    }

    fn new(layout: Layout) -> Self {
        let stacks = vec![Default::default(); layout.len()];
        Self { stacks, layout }
    }

    fn stack_row(&mut self, row: CrateRow) {
        for (pos, krate) in row.into_iter().enumerate() {
            krate.map_or((), |krate| self.stacks[pos].push(krate));
        }
    }

    fn get_stacks_mut(&mut self, route: &RouteChecked) -> StackPairMut<'_> {
        let [orig, dest] = self
            .stacks
            .get_many_mut([route.orig(), route.dest()])
            .expect("out-of-bounds route origin or destination");
        StackPairMut { orig, dest }
    }

    fn lift_pos(&self, lift: &LiftChecked) -> usize {
        self.stacks[lift.route().orig()].len() - lift.moves()
    }

    fn lift_check(&self, lift: LiftUnchecked) -> Result<LiftChecked, RouteError> {
        lift.check(&self.layout)
    }

    fn lift_params(&mut self, lift: LiftUnchecked) -> Result<LiftParams<'_>, RouteError> {
        let lift = self.lift_check(lift)?;
        let pos = self.lift_pos(&lift);
        let pair = self.get_stacks_mut(lift.route());
        Ok(LiftParams { pair, pos })
    }

    fn lift_rev(&mut self, lift: LiftUnchecked) -> Result<(), RouteError> {
        let LiftParams { pair, pos } = self.lift_params(lift)?;
        pair.dest.extend(pair.orig.lift_rev(pos));
        Ok(())
    }

    pub fn try_lifts_rev(&mut self, mut lifts: Lifts) -> Result<(), RouteError> {
        lifts.try_for_each(|lift| self.lift_rev(lift))
    }

    fn lift(&mut self, lift: LiftUnchecked) -> Result<(), RouteError> {
        let LiftParams { pair, pos } = self.lift_params(lift)?;
        pair.dest.extend(pair.orig.lift(pos));
        Ok(())
    }

    pub fn try_lifts(&mut self, mut lifts: Lifts) -> Result<(), RouteError> {
        lifts.try_for_each(|lift| self.lift(lift))
    }

    fn top_row(&self) -> impl Iterator<Item = Option<&'_ Crate>> {
        self.stacks.iter().map(|stack| stack.last())
    }

    pub fn collect_top_row<I: FromIterator<char>>(&self) -> I {
        let top_row = self.top_row();
        let chars = top_row.map(|t| t.map(Into::into).unwrap_or(' '));
        I::from_iter(chars)
    }
}

impl Extend<CrateRow> for Platform {
    fn extend<I: IntoIterator<Item = CrateRow>>(&mut self, iter: I) {
        iter.into_iter().for_each(|row| self.stack_row(row));
    }
}

#[derive(Debug, Default)]
struct Layout {
    layout: HashMap<usize, usize>,
}

impl Layout {
    pub fn len(&self) -> usize {
        self.layout.len()
    }

    fn map(&self) -> &HashMap<usize, usize> {
        &self.layout
    }
}

impl FromStr for Layout {
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

#[derive(Debug)]
struct LayoutPeeker<R: Read>(LinePeeker<R>);

impl<R: Read> LayoutPeeker<R> {
    fn into_inner(self) -> LinePeeker<R> {
        self.0
    }

    fn layout(mut self) -> (Layout, LinePeeker<R>) {
        let layout = self.next().unwrap_or_default();
        let peeker = self.into_inner();
        (layout, peeker)
    }
}

impl<R: Read> From<LinePeeker<R>> for LayoutPeeker<R> {
    fn from(peeker: LinePeeker<R>) -> Self {
        Self(peeker)
    }
}

impl<R: Read> ParseControlFlow for LayoutPeeker<R> {
    type Item = Layout;
    type ParseError = StacksLayoutParseError;
}

impl<R: Read> LinesParseIfOk for LayoutPeeker<R> {
    type InnerIter = Lines<BufReader<R>>;
    type Peekable<'s> = &'s mut PeekableLines<R> where Self: 's;

    fn peekable(&mut self) -> Self::Peekable<'_> {
        self.0.peekable()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> Iterator for LayoutPeeker<R> {
    type Item = Layout;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_if_ok()
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    const SAMPLE: &str = "    [S] [C]         [Z]            
[F] [J] [P]         [T]     [N]    
[G] [H] [G] [Q]     [G]     [D]    
[V] [V] [D] [G] [F] [D]     [V]    
[R] [B] [F] [N] [N] [Q] [L] [S]    
[J] [M] [M] [P] [H] [V] [B] [B] [D]
[L] [P] [H] [D] [L] [F] [D] [J] [L]
[D] [T] [V] [M] [J] [N] [F] [M] [G]
 1   2   3   4   5   6   7   8   9 

move 3 from 4 to 6
move 1 from 5 to 8
move 3 from 7 to 3
move 4 from 5 to 7
move 1 from 7 to 8
move 3 from 9 to 4
move 2 from 8 to 2
move 4 from 4 to 5
move 2 from 5 to 1
move 2 from 5 to 6
move 7 from 8 to 1
move 9 from 3 to 9
move 11 from 6 to 5";

    fn test_drawing() -> (Platform, Lifts) {
        let input = Cursor::new(SAMPLE);
        drawing(input)
    }

    #[test]
    fn solve_sample_lift1() {
        let (mut platform, lifts) = test_drawing();
        platform.try_lifts_rev(lifts).unwrap();
        let answer = platform.collect_top_row::<String>();
        assert_eq!("MFHDVFL M", answer);
    }

    #[test]
    fn solve_sample_lift2() {
        let (mut platform, lifts) = test_drawing();
        platform.try_lifts(lifts).unwrap();
        let answer = platform.collect_top_row::<String>();
        assert_eq!("NNHDGFH L", answer);
    }
}
