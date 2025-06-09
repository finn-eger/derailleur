#![cfg(feature = "std")]

use std::path::Path;

use csv::ReaderBuilder;
use derailleur::avec::{FromRecord, FromRecords};

#[test]
fn decode_slice_cycling() {
    const PATH: &str = "fixtures/afternoon-ride.fit";
    let data = std::fs::read(PATH).unwrap();
    let mut validator = Validator::new(PATH);
    derailleur::avec::decode_slice(&data, &mut validator).unwrap();
}

#[test]
fn decode_slice_running() {
    const PATH: &str = "fixtures/morning-trail-run.fit";
    let data = std::fs::read(PATH).unwrap();
    let mut validator = Validator::new(PATH);
    derailleur::avec::decode_slice(&data, &mut validator).unwrap();
}

#[test]
fn decode_slice_running_course() {
    const PATH: &str = "fixtures/trail-run-course.fit";
    let data = std::fs::read(PATH).unwrap();
    let mut validator = Validator::new(PATH);
    derailleur::avec::decode_slice(&data, &mut validator).unwrap();
}

#[test]
fn decode_reader_cycling() {
    const PATH: &str = "fixtures/afternoon-ride.fit";
    let mut file = std::fs::File::open(PATH).unwrap();
    let mut validator = Validator::new(PATH);
    derailleur::avec::decode_reader(&mut file, &mut validator).unwrap();
}

#[test]
fn decode_reader_running() {
    const PATH: &str = "fixtures/morning-trail-run.fit";
    let mut file = std::fs::File::open(PATH).unwrap();
    let mut validator = Validator::new(PATH);
    derailleur::avec::decode_reader(&mut file, &mut validator).unwrap();
}

#[test]
fn decode_reader_running_course() {
    const PATH: &str = "fixtures/trail-run-course.fit";
    let mut file = std::fs::File::open(PATH).unwrap();
    let mut validator = Validator::new(PATH);
    derailleur::avec::decode_reader(&mut file, &mut validator).unwrap();
}

struct Validator(Vec<String>, Vec<Vec<String>>, Option<u8>);

impl Validator {
    fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().with_extension("csv");

        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_path(path)
            .unwrap();

        let expected: Vec<Vec<String>> = reader
            .records()
            .map(|r| r.unwrap().iter().map(|f| f.to_string()).collect())
            .collect();

        Self(vec![], expected, None)
    }

    fn validate_field(&mut self, field: u8) {
        if Some(field) != self.2 {
            self.2 = Some(field);
            assert_eq!(self.0.remove(0), field.to_string())
        }
    }
}

impl FromRecords for Validator {
    fn add_record(&mut self, id: u16) -> Option<&mut dyn FromRecord> {
        self.0 = self.1.remove(0);
        assert_eq!(self.0.remove(0), id.to_string());
        Some(self)
    }
}

impl FromRecord for Validator {
    fn add_u8(&mut self, field: u8, value: u8) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_u16(&mut self, field: u8, value: u16) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_u32(&mut self, field: u8, value: u32) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_u64(&mut self, field: u8, value: u64) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_i8(&mut self, field: u8, value: i8) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_i16(&mut self, field: u8, value: i16) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_i32(&mut self, field: u8, value: i32) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_i64(&mut self, field: u8, value: i64) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_f32(&mut self, field: u8, value: f32) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
    fn add_f64(&mut self, field: u8, value: f64) {
        self.validate_field(field);
        assert_eq!(self.0.remove(0), value.to_string());
    }
}
