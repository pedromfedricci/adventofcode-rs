#![feature(iter_array_chunks)]
#![feature(iterator_try_collect)]
#![feature(get_many_mut)]
#![feature(array_windows)]

use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Lines, Read},
    iter::Peekable,
    ops::{
        ControlFlow::{self, Break, Continue},
        DerefMut,
    },
    str::FromStr,
};

pub fn input(default: &str) -> io::Result<File> {
    let path = env::args_os().into_iter().nth(1);
    let Some(path) = path else { return fallback(default) };
    File::open(path)
}

fn fallback(filename: &str) -> io::Result<File> {
    let root = env::var_os("CARGO_MANIFEST_DIR");
    let path = root.map_or_else(
        || filename.into(),
        |mut path| {
            path.push(format!("/inputs/{filename}"));
            path
        },
    );
    File::open(path)
}

pub trait LinesParseMap: LinesParse {
    type Result;

    fn map(&self, res: Result<Self::Item, Self::Error>) -> Self::Result;

    fn parse_next_map(&mut self) -> Option<Self::Result> {
        let res = self.parse_next()?;
        Some(self.map(res))
    }
}

pub trait LinesParse: ParseControlFlow {
    type Error: From<io::Error> + From<Self::ParseError>;
    type Lines<'s>: Iterator<Item = io::Result<String>>
    where
        Self: 's;

    fn lines(&mut self) -> Self::Lines<'_>;

    fn every_line(&mut self);

    fn parse_next(&mut self) -> Option<Result<Self::Item, Self::Error>> {
        loop {
            self.every_line();
            let res = match self.lines().next()? {
                Ok(ref line) => match Self::parse(line) {
                    Continue(_) => continue,
                    Break(Ok(item)) => Ok(item),
                    Break(Err(err)) => Err(err.into()),
                },
                Err(err) => Err(err.into()),
            };
            return Some(res);
        }
    }
}

pub trait LinesParseIfOk: ParseControlFlow {
    type InnerIter: Iterator<Item = Result<String, io::Error>>;
    type Peekable<'s>: DerefMut<Target = Peekable<Self::InnerIter>>
    where
        Self: 's;

    fn peekable(&mut self) -> Self::Peekable<'_>;

    fn every_line(&mut self);

    #[rustfmt::skip]
    fn parse_next_if_ok(&mut self) -> Option<Self::Item> {
        loop {
            self.every_line();
            let flow = match self.peekable().deref_mut().peek()? {
                Ok(ref line) => match Self::parse(line) {
                    Continue(_) => Continue(()),
                    Break(Ok(item)) => Break(Some(item)),
                    Break(Err(_)) => return None,
                },
                Err(_) => return None,
            };
            self.peekable().deref_mut().next();
            if let Break(item) = flow { return item }
        }
    }
}

pub trait ParseControlFlow {
    type Item: FromStr<Err = Self::ParseError>;
    type ParseError;

    fn parse(s: &str) -> ControlFlow<Result<Self::Item, Self::ParseError>, ()> {
        let s = s.trim();
        match s.is_empty() {
            true => Continue(()),
            false => Break(s.parse()),
        }
    }
}

#[derive(Debug)]
pub struct LineReader<R> {
    pos: Position,
    lines: Lines<BufReader<R>>,
}

impl<R: Read> LineReader<R> {
    pub fn new(read: R) -> Self {
        Self::with_position(read, 0)
    }

    pub fn with_position(read: R, pos: usize) -> Self {
        let lines = BufReader::new(read).lines();
        let pos = Position(pos);
        Self { lines, pos }
    }
}

impl<R> LineReader<R> {
    #[inline]
    pub fn lines(&mut self) -> &mut Lines<BufReader<R>> {
        &mut self.lines
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos.curr()
    }

    #[inline]
    pub fn advance_pos(&mut self) {
        self.pos.advance()
    }
}

pub type PeekableLines<R> = Peekable<Lines<BufReader<R>>>;

#[derive(Debug)]
pub struct LinePeeker<R: Read> {
    pos: Position,
    peeker: PeekableLines<R>,
}

impl<R: Read> LinePeeker<R> {
    pub fn new(read: R) -> Self {
        Self::with_position(read, 0)
    }

    pub fn with_position(read: R, pos: usize) -> Self {
        let peeker = BufReader::new(read).lines().peekable();
        let pos = Position(pos);
        Self { peeker, pos }
    }

    pub fn peekable(&mut self) -> &mut PeekableLines<R> {
        &mut self.peeker
    }
}

impl<R: Read> LinePeeker<R> {
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos.curr()
    }

    #[inline]
    pub fn advance_pos(&mut self) {
        self.pos.advance()
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Position(usize);

impl Position {
    fn curr(&self) -> usize {
        self.0
    }

    fn advance(&mut self) {
        self.0 += 1;
    }
}

pub mod day01;
pub mod day02;
pub mod day03;
pub mod day04;
pub mod day05;
pub mod day06;
