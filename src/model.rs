#![expect(dead_code)]

use core::fmt::Display;

use egui::Color32;

#[derive(Debug, Clone)]
pub struct Lineage {
    pub name: &'static str, // e.g., "Nintendo Home"
    pub maker: Maker,
    pub color: Color32,
    /// Ordered consoles -> implicit succession for most timelines
    pub consoles: &'static [Console],
}

#[derive(Debug, Clone)]
pub struct Console {
    pub name: &'static str,       // "Super Nintendo Entertainment System"
    pub short_name: &'static str, // "SNES"
    pub generation: Option<u8>,
    pub releases: &'static [Release],     // per region
    pub milestones: &'static [Milestone], // end of prod, last game, etc.
    pub measures: &'static [Measure],     // lifetime snapshots
    pub series: &'static [Series],        // per-year data
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Maker {
    Nintendo,
    Sony,
    Microsoft,
    Sega,
    Valve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Global,
    JP,
    NA,
    EU,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialDate {
    Year { y: i32 },
    YearMonth { y: i32, m: u8 },
    YearMonthDay { y: i32, m: u8, d: u8 },
}

impl PartialDate {
    pub fn year(&self) -> i32 {
        match *self {
            Self::Year { y } | Self::YearMonth { y, .. } | Self::YearMonthDay { y, .. } => y,
        }
    }

    pub fn cmp_key(&self) -> (i32, u8, u8) {
        match *self {
            Self::Year { y } => (y, 1, 1),
            Self::YearMonth { y, m } => (y, m, 1),
            Self::YearMonthDay { y, m, d } => (y, m, d),
        }
    }
}

impl Display for PartialDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Year { y } => write!(f, "{y}"),
            Self::YearMonth { y, m } => write!(f, "{y}-{m:02}"),
            Self::YearMonthDay { y, m, d } => write!(f, "{y}-{m:02}-{d:02}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneKind {
    EndOfProduction,
    LastFirstPartyGame,
    LastThirdPartyGame,
    LastUnitSold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesKind {
    UnitsSoldPerYear,
    GamesReleasedPerYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureKind {
    UnitsSold,     // lifetime
    GamesReleased, // lifetime
    AttachRate,    // percentage *100? up to you
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimePoint {
    pub year: i32,
    pub value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Estimate<T> {
    pub point: T,
    pub low: Option<T>,
    pub high: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub label: &'static str,       // “Nintendo IR 2025Q2”
    pub url: Option<&'static str>, // keep optional for offline sources
}

#[derive(Debug, Clone)]
pub struct Release {
    pub region: Region,
    pub date: PartialDate,
    pub source: Option<Source>,
}

#[derive(Debug, Clone)]
pub struct Milestone {
    pub kind: MilestoneKind,
    pub date: PartialDate,
    pub region: Option<Region>,
    pub source: Option<Source>,
}

/// Scalar measures (snapshots)
#[derive(Debug, Clone)]
pub struct Measure {
    pub kind: MeasureKind,
    pub value: Estimate<u64>,
    pub region: Region,
    pub as_of_year: Option<i32>,
    pub source: Option<Source>,
}

/// Time series
#[derive(Debug, Clone)]
pub struct Series {
    pub kind: SeriesKind,
    pub points: &'static [TimePoint],
    pub region: Region,
    pub source: Option<Source>,
}
