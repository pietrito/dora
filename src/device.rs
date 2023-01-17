use crate::errors::{Error, Result};
use crate::globals::{DEFAULT_SECTOR_SIZE, GB_SIZE};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::path::PathBuf;

pub(crate) struct Device {
    fd: BufReader<File>,
    current: u64,
    status: bool,
    sector_size: usize,
}

impl Device {
    pub fn new(
        path: PathBuf,
        sector_size: Option<usize>,
        seek: Option<String>,
        status: Option<bool>,
    ) -> Result<Self> {
        let file = File::open(path).map_err(Error::FileOpen)?;
        let sector_size = sector_size.unwrap_or(DEFAULT_SECTOR_SIZE);
        let mut buf_reader = BufReader::with_capacity(sector_size, file);
        let status = status.unwrap_or(false);

        // Start at `GB_SEEK` instead of the begining of the device
        let current = match seek {
            Some(seek) => {
                let seek = Self::parse_size(seek)?;

                buf_reader
                    .seek(SeekFrom::Start(seek))
                    .map_err(|e| Error::SeekFailed(e))?
            }
            None => 0,
        };

        Ok(Self {
            fd: buf_reader,
            status,
            current,
            sector_size,
        })
    }

    pub fn get_size(&mut self) -> Result<u64> {
        let current = self
            .fd
            .seek(SeekFrom::Current(0))
            .map_err(|e| Error::SeekFailed(e))?;

        // Get device size, print it, rewind
        let dev_size = self.fd.seek(SeekFrom::End(0)).map_err(Error::SeekFailed)?;
        self.fd
            .seek(SeekFrom::Start(current))
            .map_err(Error::RewindFailed)?;
        Ok(dev_size)
    }

    pub fn find_first_non_zero(&mut self) -> Result<usize> {
        let mut buffer = vec![0; self.sector_size];

        println!(
            "Starting looking for data from {}Gb",
            self.current / GB_SIZE
        );

        loop {
            if self.status {
                print!("\rRead {}Gb\t", self.current / GB_SIZE);
                std::io::stdout()
                    .flush()
                    .map_err(|e| Error::FlushError(e))?;
            }

            let nbytes = self.fd.read(&mut buffer).map_err(|e| Error::ReadError(e))?;

            if buffer.iter().any(|byte| byte != &0) {
                let offset = self.current / GB_SIZE;
                println!("Found non-empty buffer at @{}Gb", offset);

                break;
            }

            if nbytes != self.sector_size {
                println!("The device is full of zeroes!!");
                break;
            }

            // Seek to next Gb
            let next = GB_SIZE - (self.current % GB_SIZE);
            self.current = self
                .fd
                .seek(SeekFrom::Current(next as i64))
                .map_err(|e| Error::SeekFailed(e))?;
        }

        Ok(0)
    }

    fn parse_size(nb: String) -> Result<u64> {
        match nb.ends_with('B') {
            true => {
                if nb.ends_with("GB") {
                    let parsed = nb[..nb.find("GB").unwrap()]
                        .parse::<u64>()
                        .map_err(|_| Error::ParseError)?;
                    return Ok(parsed * GB_SIZE);
                }

                let parsed = nb[..nb.find('B').unwrap()]
                    .parse::<u64>()
                    .map_err(|_| Error::ParseError)?;

                return Ok(parsed);
            }
            false => {
                return Ok(nb.parse::<u64>().map_err(|_| Error::ParseError)?);
            }
        }
    }
}
