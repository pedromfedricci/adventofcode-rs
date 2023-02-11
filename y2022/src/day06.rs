use std::{
    fs::File,
    io::{self, BufReader, Read},
    slice::ArrayWindows,
};

use itertools::Itertools;

pub fn day06_file() -> io::Result<File> {
    super::input("day06")
}

#[derive(Debug)]
pub struct DataSource(Vec<char>);

impl DataSource {
    pub fn new<R: Read>(read: R) -> io::Result<Self> {
        let mut buf = String::new();
        BufReader::new(read).read_to_string(&mut buf)?;
        let chars = buf.chars().collect();
        Ok(Self(chars))
    }

    pub fn windows<const SIZE: usize>(&self) -> DataWindows<'_, SIZE> {
        DataWindows::new(&self.0[..])
    }
}

#[derive(Debug)]
pub struct DataWindows<'source, const SIZE: usize> {
    windows: ArrayWindows<'source, char, SIZE>,
}

impl<'source, const SIZE: usize> DataWindows<'source, SIZE> {
    fn new(source: &'source [char]) -> Self {
        let windows = source.array_windows();
        Self { windows }
    }
}

impl<'source, const SIZE: usize> Iterator for DataWindows<'source, SIZE> {
    type Item = &'source [char; SIZE];

    fn next(&mut self) -> Option<Self::Item> {
        self.windows.next()
    }
}

fn marker_finder<'source, I, const SIZE: usize>(mut iter: I) -> Result<usize, MarkerError>
where
    I: Iterator<Item = &'source [char; SIZE]>,
{
    iter.position(|window| window.iter().unique().count() == SIZE)
        .map(|pos| pos + SIZE)
        .ok_or(MarkerError)
}

const PACKET_MARKER_SIZE: usize = 4;

pub fn start_of_packet<'source, I>(iter: I) -> Result<usize, MarkerError>
where
    I: Iterator<Item = &'source [char; PACKET_MARKER_SIZE]>,
{
    marker_finder::<I, PACKET_MARKER_SIZE>(iter)
}

const MESSAGE_MARKER_SIZE: usize = 14;

pub fn start_of_message<'source, I>(iter: I) -> Result<usize, MarkerError>
where
    I: Iterator<Item = &'source [char; MESSAGE_MARKER_SIZE]>,
{
    marker_finder::<I, MESSAGE_MARKER_SIZE>(iter)
}

#[derive(Debug, thiserror::Error)]
#[error("No marker found in the data stream")]
pub struct MarkerError;
