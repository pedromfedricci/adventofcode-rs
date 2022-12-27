use crate::{LineReader, LinesParse, LinesParseMap, ParseControlFlow};

use std::{
    collections::HashSet,
    fs::File,
    hash::{Hash, Hasher},
    io::{self, BufReader, Lines, Read},
    str::FromStr,
};

pub fn day03_file() -> io::Result<File> {
    super::input("day03")
}

#[derive(Debug, Copy, Clone)]
pub struct Priority {
    inner: u32,
}

impl Priority {
    // PANIC: char::to_digit will panic if provided radix is greater than 36;
    const RADIX: u32 = 36;
    const LOWERCASE_OFFSET: u32 = 9;
    const UPPERCASE_OFFSET: u32 = 17;

    pub fn into_inner(self) -> u32 {
        self.inner
    }

    pub fn new(c: char) -> Result<Self, ItemError> {
        match (c.is_ascii_alphabetic(), c.to_digit(Self::RADIX)) {
            (true, Some(digit)) => {
                let inner = if c.is_ascii_lowercase() {
                    digit - Self::LOWERCASE_OFFSET
                } else {
                    digit + Self::UPPERCASE_OFFSET
                };
                Ok(Self { inner })
            }
            _ => Err(ItemError(c)),
        }
    }
}

impl TryFrom<char> for Priority {
    type Error = ItemError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Item {
    value: char,
    priority: Priority,
}

impl Item {
    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn value(&self) -> char {
        self.value
    }

    pub fn new(value: char) -> Result<Self, ItemError> {
        let priority = Priority::try_from(value)?;
        Ok(Self { value, priority })
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Eq for Item {}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl TryFrom<char> for Item {
    type Error = ItemError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
#[error("invalid item type: `{0}`, must be either in a-z or A-Z")]
pub struct ItemError(char);

#[derive(Debug, Default)]
struct Compartment {
    set: HashSet<Item>,
}

impl Compartment {
    fn insert_next(&mut self, value: Option<char>) -> Result<Option<()>, ItemError> {
        let Some(value) = value else { return Ok(None) };
        if !value.is_whitespace() {
            self.set.insert(Item::try_from(value)?);
        }
        Ok(Some(()))
    }
}

#[derive(Debug, Default)]
pub struct Rucksack(Compartment, Compartment);

impl Rucksack {
    pub fn common(&self) -> HashSet<Item> {
        &self.0.set & &self.1.set
    }

    pub fn all(&self) -> HashSet<Item> {
        &self.0.set | &self.1.set
    }

    pub fn common_sum(&self) -> u32 {
        self.common().into_iter().map(|i| i.priority().into_inner()).sum()
    }
}

impl FromStr for Rucksack {
    type Err = ItemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rucksack = Rucksack::default();
        let mut chars = s.trim().chars();
        loop {
            let next = rucksack.0.insert_next(chars.next())?;
            let back = rucksack.1.insert_next(chars.next_back())?;
            if let (None, None) = (next, back) {
                break;
            }
        }
        Ok(rucksack)
    }
}

#[derive(Debug)]
pub struct RucksackReader<R>(LineReader<R>);

impl<R: Read> RucksackReader<R> {
    pub fn new(read: R) -> Self {
        let reader = LineReader::new(read);
        Self(reader)
    }

    pub fn common_sum(self) -> Result<u32, RucksackError> {
        self.into_iter().try_fold(0, |mut sum, rucksack| {
            sum += rucksack?.common_sum();
            Ok(sum)
        })
    }
}

impl<R: Read> Iterator for RucksackReader<R> {
    type Item = Result<Rucksack, RucksackError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_map()
    }
}

impl<R> ParseControlFlow for RucksackReader<R> {
    type Item = Rucksack;
    type ParseError = ItemError;
}

impl<R: Read> LinesParse for RucksackReader<R> {
    type Error = RucksackErrorSource;
    type Lines = Lines<BufReader<R>>;

    fn lines(&mut self) -> &mut Self::Lines {
        self.0.lines()
    }

    fn every_line(&mut self) {
        self.0.advance_pos()
    }
}

impl<R: Read> LinesParseMap for RucksackReader<R> {
    type Result = Result<Rucksack, RucksackError>;

    fn map(&self, res: Result<Self::Item, Self::Error>) -> Self::Result {
        res.map_err(|err| RucksackError::new(self.0.pos(), err))
    }
}

#[derive(Debug)]
pub struct RucksackGroup {
    id: usize,
    badge: Item,
}

impl RucksackGroup {
    fn new(id: usize, group: Vec<Rucksack>) -> Result<Self, GroupError> {
        let mut iter = group.iter();
        let mut items = iter.next().ok_or_else(|| GroupError::empty(id))?.all();
        iter.for_each(|rucksack| items = &items & &rucksack.all());
        let mut items = items.into_iter();
        let badge = items.next().ok_or_else(|| GroupError::missing(id))?;
        let None = items.next() else { return Err(GroupError::too_many(id)) };
        Ok(Self { id, badge })
    }

    pub fn badge(&self) -> Item {
        self.badge
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

pub struct RucksackGroupReader<R> {
    size: usize,
    pos: usize,
    rucksacks: RucksackReader<R>,
}

impl<R: Read> RucksackGroupReader<R> {
    pub fn new(read: R, size: usize) -> Self {
        let rucksacks = RucksackReader::new(read);
        Self { rucksacks, size, pos: 1 }
    }

    pub fn badges_sum(self) -> Result<u32, GroupError> {
        self.into_iter().try_fold(0, |mut sum, group| {
            sum += group?.badge().priority().into_inner();
            Ok(sum)
        })
    }
}

impl<R: Read> Iterator for RucksackGroupReader<R> {
    type Item = Result<RucksackGroup, GroupError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut group = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            match self.rucksacks.next() {
                Some(Ok(rucksack)) => group.push(rucksack),
                Some(Err(err)) => return Some(Err(GroupError::rucksack(self.pos, err))),
                None if group.is_empty() => return None,
                None => continue,
            }
        }
        let group = RucksackGroup::new(self.pos, group);
        self.pos += 1;
        Some(group)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error at line: {pos}, {source}")]
pub struct RucksackError {
    pos: usize,
    #[source]
    source: RucksackErrorSource,
}

impl RucksackError {
    fn new(pos: usize, source: RucksackErrorSource) -> Self {
        Self { pos, source }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RucksackErrorSource {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Item(#[from] ItemError),
}

#[derive(Debug, thiserror::Error)]
#[error("error with elf group: {id}, {source}")]
pub struct GroupError {
    id: usize,
    #[source]
    source: GroupErrorSource,
}

impl GroupError {
    fn empty(id: usize) -> Self {
        let source = GroupErrorSource::Empty(());
        Self { id, source }
    }

    fn missing(id: usize) -> Self {
        let source = GroupErrorSource::Missing(());
        Self { id, source }
    }

    fn too_many(id: usize) -> Self {
        let source = GroupErrorSource::TooMany(());
        Self { id, source }
    }

    fn rucksack(id: usize, err: RucksackError) -> Self {
        let source = GroupErrorSource::Rucksack(err);
        Self { id, source }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GroupErrorSource {
    #[error("group is missing it's badge")]
    Missing(()),
    #[error("group is empty")]
    Empty(()),
    #[error("group has more than one badge")]
    TooMany(()),
    #[error(transparent)]
    Rucksack(#[from] RucksackError),
}
