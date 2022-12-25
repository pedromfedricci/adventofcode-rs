use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Lines, Read},
    ops::ControlFlow::{self, Break, Continue},
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
    type Lines: Iterator<Item = Result<String, io::Error>>;

    fn lines(&mut self) -> &mut Self::Lines;

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
    pos: usize,
    lines: Lines<BufReader<R>>,
}

impl<R: Read> LineReader<R> {
    pub fn new(read: R) -> Self {
        Self::with_position(read, 0)
    }

    pub fn with_position(read: R, pos: usize) -> Self {
        let lines = BufReader::new(read).lines();
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
        self.pos
    }

    #[inline]
    pub fn advance_pos(&mut self) {
        self.pos += 1;
    }
}

pub mod day01;
pub mod day02;

