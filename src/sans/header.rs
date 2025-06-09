//! States processing document and record headers.

use either::Either::{self, Left, Right};
use tartan_bitfield::bitfield;
use thiserror::Error;
use zerocopy::FromBytes;

use super::definition::{Definition, DefinitionAlt};

/// An error advancing over a document header.
#[derive(Debug, Error)]
pub enum DocumentHeaderError {
    /// Incorrect filetype marker.
    #[error("Incorrect file type marker.")]
    NotFitData,
    /// Unknown header length.
    #[error("Unknown header length ({0}).")]
    UnknownHeaderLength(u8),
}

/// State token to decode a document header.
#[derive(Debug)]
pub struct DocumentHeader;

impl DocumentHeader {
    /// Transition to another state by decoding a document header.
    ///
    /// Returns the number of record bytes in this document, and a successor
    /// state token.
    pub fn advance(
        r: [u8; 12],
    ) -> Result<(u32, Either<ExtendedDocumentHeader, RecordHeader>), DocumentHeaderError> {
        #[repr(C, packed)]
        #[derive(FromBytes)]
        struct FileHeader {
            header_size: u8,
            protocol_version: u8,
            profile_version: u16,
            data_size: u32,
            data_type: [u8; 4],
        }

        let FileHeader {
            header_size,
            data_size,
            data_type,
            ..
        } = zerocopy::transmute!(r);

        if &data_type != b".FIT" {
            Err(DocumentHeaderError::NotFitData)?;
        }

        let successor = match header_size {
            14 => Left(ExtendedDocumentHeader(())),
            12 => Right(RecordHeader(())),
            _ => Err(DocumentHeaderError::UnknownHeaderLength(header_size))?,
        };

        Ok((data_size, successor))
    }
}

/// State token to decode additional bytes of an extended document header.
#[derive(Debug)]
pub struct ExtendedDocumentHeader(pub(super) ());

impl ExtendedDocumentHeader {
    /// Transition to another state by decoding the additional bytes of an
    /// extended document header.
    ///
    /// Returns the successor state token.
    pub fn advance(self, _r: [u8; 2]) -> RecordHeader {
        RecordHeader(())
    }
}

/// An error advancing over a record header.
#[derive(Debug, Error)]
pub enum RecordHeaderError {
    /// Found developer data (not yet supported).
    #[error("Found developer data.")]
    DeveloperData,
}

/// State token to decode a record header.
#[derive(Debug)]
pub struct RecordHeader(pub(super) ());

impl RecordHeader {
    /// Transition to another state by decoding a record header.
    ///
    /// Returns the local message number, a successor state token, and for
    /// record headers, the time offset if present.
    pub fn advance(
        self,
        r: [u8; 1],
    ) -> Result<(u8, Either<Definition, (Option<u8>, DefinitionAlt)>), RecordHeaderError> {
        let r = r[0];

        bitfield! {
            struct RecordHeader(u8) {
                [7] is_compressed,
            }
        }

        let header = RecordHeader(r);

        if header.is_compressed() {
            bitfield! {
                struct CompressedHeader(u8) {
                    [0..5] time_offset: u8,
                    [5..7] local_message: u8,
                }
            }

            let header = CompressedHeader(r);

            let local_message = header.local_message();
            let time_offset = header.time_offset();

            let successor = Right((Some(time_offset), DefinitionAlt(())));

            Ok((local_message, successor))
        } else {
            bitfield! {
                struct NormalHeader(u8) {
                    [0..4] local_message: u8,
                    [5] is_developer,
                    [6] is_definition,
                }
            }

            let header = NormalHeader(r);

            let local_message = header.local_message();
            if header.is_developer() {
                Err(RecordHeaderError::DeveloperData)?;
            }

            let successor = if header.is_definition() {
                Left(Definition(()))
            } else {
                Right((None, DefinitionAlt(())))
            };

            Ok((local_message, successor))
        }
    }
}
