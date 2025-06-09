#![allow(dead_code, unused)]
#![cfg(all(feature = "derive", feature = "std"))]

use std::{fs::read_to_string, path::Path};

use derailleur::avec::{FromRecord, FromRecords};
use tinyvec::ArrayVec;
use zerocopy::TryFromBytes;

#[test]
fn decode_slice_cycling() {
    const PATH: &str = "fixtures/afternoon-ride.fit";
    let data = std::fs::read(PATH).unwrap();
    let mut records = ActivityRecordSet::default();
    derailleur::avec::decode_slice(&data, &mut records).unwrap();

    let path = Path::new(PATH).with_extension("dbg");
    let debug = read_to_string(path).unwrap();
    assert_eq!(format!("{records:?}"), debug.trim());
}

#[derive(Debug, Default, FromRecords)]
struct ActivityRecordSet {
    #[record(0)]
    file_id: Option<FileId>,
    #[record(20)]
    records: Vec<Record>,
}

#[derive(Debug, Default, FromRecord)]
struct FileId {
    #[field(3)]
    serial_number: Option<u32>,
    #[field(4)]
    time_created: Option<u32>,
    #[field(1)]
    manufacturer: Option<u16>,
    #[field(2)]
    product: Option<u16>,
    #[field(0)]
    type_: Option<u8>,
}

#[derive(Debug, Default, FromRecord)]
struct Record {
    #[field(time)]
    time_offset: Option<u8>,
    #[field(253)]
    timestamp: Option<u32>,
    #[field(0)]
    position_lat: Option<i32>,
    #[field(1)]
    position_long: Option<i32>,
    #[field(2)]
    altitude: Option<u16>,
    #[field(5)]
    distance: Option<u32>,
    #[field(6)]
    speed: Option<u16>,
    #[field(13)]
    temperature: Option<i8>,
}

#[test]
fn decode_slice_running_course() {
    const PATH: &str = "fixtures/trail-run-course.fit";
    let data = std::fs::read(PATH).unwrap();
    let mut records = CourseRecordSet::default();
    derailleur::avec::decode_slice(&data, &mut records).unwrap();

    let path = Path::new(PATH).with_extension("dbg");
    let debug = read_to_string(path).unwrap();
    assert_eq!(format!("{records:?}"), debug.trim());
}

#[derive(Debug, Default, FromRecords)]
struct CourseRecordSet {
    #[record(0)]
    file_id: Option<FileId>,
    #[record(31)]
    course: Option<Course>,
    #[record(32)]
    course_points: Vec<CoursePoint>,
}

#[derive(Debug, Default, FromRecord)]
struct Course {
    #[field(5, |v, c: u8| v.push(c))]
    name: Vec<u8>,
}

impl Course {
    fn name(&self) -> Option<&str> {
        if self.name.len() != 0 {
            std::str::from_utf8(&self.name).ok()
        } else {
            None
        }
    }
}

#[derive(Debug, Default, FromRecord)]
struct CoursePoint {
    #[field(1)]
    timestamp: Option<u32>,
    #[field(2)]
    position_lat: Option<i32>,
    #[field(3)]
    position_long: Option<i32>,
    #[field(4)]
    distance: Option<u32>,
    #[field(5, |p, x: u8| {
        if let Ok(x) = zerocopy::try_transmute!(x) {
            *p = Some(x);
        }
    })]
    type_: Option<CoursePointType>,
    #[field(6, |(a, i), c: u8| {
        if *i <= 10 {
            a[*i] = c;
            *i += 1;
        }
    })]
    name: ([u8; 10], usize),
}

#[repr(u8)]
#[derive(Debug, Default, TryFromBytes)]
enum CoursePointType {
    #[default]
    Generic = 0,
    Left = 6,
    Right = 7,
    SlightLeft = 19,
    SlightRight = 21,
}

impl CoursePoint {
    fn name(&self) -> Option<&str> {
        if self.name.1 != 0 {
            std::str::from_utf8(&self.name.0).ok()
        } else {
            None
        }
    }
}
