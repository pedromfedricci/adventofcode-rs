use crate::{LineReader, LinesParse, LinesParseMap, ParseControlFlow};

use std::{
    cmp::Ordering,
    fs::File,
    io::{self, BufReader, Lines, Read},
    ops::Add,
    str::FromStr,
};

pub fn day02_file() -> io::Result<File> {
    super::input("day02")
}

type Score = u64;
type Turns = usize;

#[derive(Debug, Copy, Clone, Default)]
pub struct Player {
    score: Score,
    wins: Turns,
    losses: Turns,
    draws: Turns,
}

impl Player {
    fn update(&mut self, shape: Shape, outcome: Outcome) {
        self.score = self.score + shape + outcome;
        match outcome {
            Outcome::Won => self.wins += 1,
            Outcome::Lost => self.losses += 1,
            Outcome::Draw => self.draws += 1,
        }
    }

    pub fn score(&self) -> Score {
        self.score
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Players(Player, Player);

impl Players {
    fn play(&mut self, shapes: Shapes) {
        let (sha0, sha1) = shapes.into();
        let (out0, out1) = Outcomes::resolve(shapes).into();
        self.0.update(sha0, out0);
        self.1.update(sha1, out1);
    }
}

#[derive(Debug)]
pub struct Game<R> {
    turn: Turns,
    players: Players,
    rounds: RowReader<R>,
}

impl<R: Read> Game<R> {
    pub fn new(read: R) -> Self {
        let (turn, players) = Default::default();
        let rounds = RowReader::new(read);
        Self { turn, players, rounds }
    }

    pub fn tournament1(mut self) -> Result<Statistics, RowError> {
        let iter = self.rounds.into_iter().map(|row| Ok(Shapes::from(row?)));
        for shapes in iter {
            self.players.play(shapes?);
        }
        Ok(Statistics::new(self.turn, self.players))
    }

    pub fn tournament2(mut self) -> Result<Statistics, RowError> {
        let iter = self.rounds.into_iter().map(|row| Ok(Strategy::from(row?)));
        for strat in iter {
            self.players.play(strat?.solve());
        }
        Ok(Statistics::new(self.turn, self.players))
    }
}

#[derive(Debug)]
pub struct Statistics {
    turns: Turns,
    players: Players,
}

impl Statistics {
    fn new(turns: Turns, players: Players) -> Self {
        Self { turns, players }
    }

    pub fn opponent(&self) -> &Player {
        &self.players.0
    }

    pub fn protagonist(&self) -> &Player {
        &self.players.1
    }

    pub fn turns(&self) -> &Turns {
        &self.turns
    }
}

#[derive(Debug, Copy, Clone)]
struct Strategy(Shape, Outcome);

impl Strategy {
    fn solve(self) -> Shapes {
        let choice = match (self.0, self.1) {
            (Shape::Rock, Outcome::Draw)
            | (Shape::Paper, Outcome::Lost)
            | (Shape::Scissors, Outcome::Won) => Shape::Rock,
            (Shape::Rock, Outcome::Won)
            | (Shape::Paper, Outcome::Draw)
            | (Shape::Scissors, Outcome::Lost) => Shape::Paper,
            (Shape::Rock, Outcome::Lost)
            | (Shape::Paper, Outcome::Won)
            | (Shape::Scissors, Outcome::Draw) => Shape::Scissors,
        };
        (self.0, choice).into()
    }
}

impl From<Row> for Strategy {
    fn from(row: Row) -> Self {
        Self(Shape::from(row.0), Outcome::from(row.1))
    }
}

#[derive(Debug, Copy, Clone)]
enum Outcome {
    Lost,
    Draw,
    Won,
}

impl Outcome {
    const LOST_SCORE: Score = 0;
    const DRAW_SCORE: Score = 3;
    const WON_SCORE: Score = 6;
}

impl From<Column1> for Outcome {
    fn from(col1: Column1) -> Self {
        match col1 {
            Column1::X => Self::Lost,
            Column1::Y => Self::Draw,
            Column1::Z => Self::Won,
        }
    }
}

impl From<Outcome> for Score {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Lost => Outcome::LOST_SCORE,
            Outcome::Draw => Outcome::DRAW_SCORE,
            Outcome::Won => Outcome::WON_SCORE,
        }
    }
}

impl Add<Outcome> for Score {
    type Output = Score;

    fn add(self, rhs: Outcome) -> Self::Output {
        self + Score::from(rhs)
    }
}

#[derive(Debug, Copy, Clone)]
struct Outcomes(Outcome, Outcome);

impl Outcomes {
    fn resolve(shapes: Shapes) -> Self {
        let out0 = shapes.against();
        let out1 = shapes.rev_against();
        Self(out0, out1)
    }
}

impl From<Outcomes> for (Outcome, Outcome) {
    fn from(outcomes: Outcomes) -> Self {
        (outcomes.0, outcomes.1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

impl Shape {
    const ROCK_SCORE: Score = 1;
    const PAPER_SCORE: Score = 2;
    const SCISSORS_SCORE: Score = 3;

    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Rock, Self::Rock)
            | (Self::Paper, Self::Paper)
            | (Self::Scissors, Self::Scissors) => Ordering::Equal,
            (Self::Rock, Self::Scissors)
            | (Self::Paper, Self::Rock)
            | (Self::Scissors, Self::Paper) => Ordering::Greater,
            (Self::Rock, Self::Paper)
            | (Self::Paper, Self::Scissors)
            | (Self::Scissors, Self::Rock) => Ordering::Less,
        }
    }

    fn against(self, other: Self) -> Outcome {
        match self.cmp(&other) {
            Ordering::Equal => Outcome::Draw,
            Ordering::Greater => Outcome::Won,
            Ordering::Less => Outcome::Lost,
        }
    }
}

impl Add<Shape> for Score {
    type Output = Score;

    fn add(self, rhs: Shape) -> Self::Output {
        self + Score::from(rhs)
    }
}

impl PartialOrd for Shape {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Shape {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl From<Shape> for Score {
    fn from(shape: Shape) -> Self {
        match shape {
            Shape::Rock => Shape::ROCK_SCORE,
            Shape::Paper => Shape::PAPER_SCORE,
            Shape::Scissors => Shape::SCISSORS_SCORE,
        }
    }
}

impl From<Column0> for Shape {
    fn from(col: Column0) -> Self {
        match col {
            Column0::A => Self::Rock,
            Column0::B => Self::Paper,
            Column0::C => Self::Scissors,
        }
    }
}

impl From<Column1> for Shape {
    fn from(col: Column1) -> Self {
        match col {
            Column1::Y => Self::Paper,
            Column1::X => Self::Rock,
            Column1::Z => Self::Scissors,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Shapes(Shape, Shape);

impl Shapes {
    fn against(self) -> Outcome {
        self.0.against(self.1)
    }

    fn rev_against(self) -> Outcome {
        self.1.against(self.0)
    }
}

impl From<Shapes> for (Shape, Shape) {
    fn from(shapes: Shapes) -> Self {
        (shapes.0, shapes.1)
    }
}

impl From<(Shape, Shape)> for Shapes {
    fn from(shapes: (Shape, Shape)) -> Self {
        Self(shapes.0, shapes.1)
    }
}

trait Column: TryFrom<char, Error = ColumnParseError> {
    const POS: usize;

    fn col_from_str(s: &str) -> Result<Self, ColumnError> {
        let mut chars = s.chars();
        let col = chars.next().ok_or(ColumnError::Empty(Self::POS))?;
        let None = chars.next() else {
            return Err(ColumnError::Larger(Self::POS));
        };
        Ok(col.try_into()?)
    }

    fn col_try_from(value: Option<&str>) -> Result<Self, ColumnError> {
        Self::col_from_str(value.ok_or(ColumnError::Empty(Self::POS))?)
    }
}

#[derive(Debug, Copy, Clone)]
enum Column0 {
    A,
    B,
    C,
}

impl Column0 {
    const POS: usize = 0;
}

impl TryFrom<char> for Column0 {
    type Error = ColumnParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(Self::A),
            'B' => Ok(Self::B),
            'C' => Ok(Self::C),
            _ => Err(Self::Error::new(Self::POS, value)),
        }
    }
}

impl Column for Column0 {
    const POS: usize = Self::POS;
}

impl FromStr for Column0 {
    type Err = ColumnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Column::col_from_str(s)
    }
}

impl TryFrom<Option<&str>> for Column0 {
    type Error = ColumnError;

    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        Column::col_try_from(value)
    }
}

#[derive(Debug, Copy, Clone)]
enum Column1 {
    Y,
    X,
    Z,
}

impl Column1 {
    const POS: usize = 1;
}

impl TryFrom<char> for Column1 {
    type Error = ColumnParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'Y' => Ok(Self::Y),
            'X' => Ok(Self::X),
            'Z' => Ok(Self::Z),
            _ => Err(Self::Error::new(Self::POS, value)),
        }
    }
}

impl Column for Column1 {
    const POS: usize = Self::POS;
}

impl FromStr for Column1 {
    type Err = ColumnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Column::col_from_str(s)
    }
}

impl TryFrom<Option<&str>> for Column1 {
    type Error = ColumnError;

    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        Column::col_try_from(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Row(Column0, Column1);

impl Row {
    const DELIMETER: &'static str = "<whitespace>";
}

impl From<Row> for Shapes {
    fn from(row: Row) -> Self {
        Self(Shape::from(row.0), Shape::from(row.1))
    }
}

impl FromStr for Row {
    type Err = RowParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cols = s.split_whitespace().into_iter().peekable();
        cols.peek().ok_or(RowParseError::Delimeter(Self::DELIMETER))?;
        let col0 = Column0::try_from(cols.next())?;
        let col1 = Column1::try_from(cols.next())?;
        Ok(Self(col0, col1))
    }
}

#[derive(Debug)]
pub struct RowReader<R>(LineReader<R>);

impl<R: Read> RowReader<R> {
    pub fn new(read: R) -> Self {
        let reader = LineReader::new(read);
        Self(reader)
    }
}

impl<R: Read> Iterator for RowReader<R> {
    type Item = Result<Row, RowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_map()
    }
}

impl<R> ParseControlFlow for RowReader<R> {
    type Item = Row;
    type ParseError = RowParseError;
}

impl<R: Read> LinesParse for RowReader<R> {
    type Error = RowErrorSource;
    type Lines<'s> = &'s mut Lines<BufReader<R>> where Self: 's;

    fn lines(&mut self) -> Self::Lines<'_> {
        self.0.lines()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> LinesParseMap for RowReader<R> {
    type Result = Result<Self::Item, RowError>;

    fn map(&self, res: Result<Self::Item, Self::Error>) -> Self::Result {
        res.map_err(|err| RowError::new(self.0.pos(), err))
    }
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum ColumnError {
    #[error(transparent)]
    Parse(#[from] ColumnParseError),
    #[error("missing value for column: {0}")]
    Empty(usize),
    #[error("value for column: {0} is too large")]
    Larger(usize),
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
#[error("invalid value: `{value}` at column: {pos}")]
pub struct ColumnParseError {
    pos: usize,
    value: char,
}

impl ColumnParseError {
    fn new(pos: usize, value: char) -> Self {
        Self { pos, value }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error at line: {pos}, {source}")]
pub struct RowError {
    pos: usize,
    #[source]
    source: RowErrorSource,
}

impl RowError {
    fn new(pos: usize, source: RowErrorSource) -> Self {
        Self { pos, source }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RowErrorSource {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] RowParseError),
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum RowParseError {
    #[error(transparent)]
    Column(#[from] ColumnError),
    #[error("delimeter: `{0}` was not found")]
    Delimeter(&'static str),
}
