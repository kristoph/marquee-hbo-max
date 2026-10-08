use crate::{net::ByteRange, Error};

const HEADER_BYTES: usize = 20;
const REFERENCE_BYTES: usize = 12;
const SIZE_MASK: u32 = 0x7fff_ffff;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Segment {
    pub offset: u64,
    pub bytes: u64,
    pub seconds: f64,
}

/// Reads a file's segment index (its `sidx` box), which lies at `index_range` in the file and
/// lists the segments that follow it.
pub(super) fn segments(index_box: &[u8], index_range: ByteRange) -> Result<Vec<Segment>, Error> {
    let short = || Error::Lacks("the whole of a segment index");
    let number = |at: usize, bytes: usize| -> Result<u64, Error> {
        let slice = index_box.get(at..at + bytes).ok_or_else(short)?;
        Ok(slice.iter().fold(0, |value, byte| (value << 8) | u64::from(*byte)))
    };
    if index_box.get(4..8) != Some(b"sidx") {
        return Err(Error::Lacks("a segment index where the manifest says"));
    }
    let wide_fields = index_box.get(8) == Some(&1);
    let timescale = number(16, 4)? as f64;
    let field = if wide_fields { 8 } else { 4 };
    let first_offset = number(HEADER_BYTES + field, field)?;
    let count_at = HEADER_BYTES + field * 2 + 2;
    let count = number(count_at, 2)? as usize;

    let mut offset = index_range.last + 1 + first_offset;
    let mut segments = Vec::with_capacity(count);
    for reference in 0..count {
        let at = count_at + 2 + reference * REFERENCE_BYTES;
        let bytes = u64::from(number(at, 4)? as u32 & SIZE_MASK);
        segments.push(Segment { offset, bytes, seconds: number(at + 4, 4)? as f64 / timescale });
        offset += bytes;
    }
    Ok(segments)
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub fn index_box(timescale: u32, references: &[(u32, u32)]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(((HEADER_BYTES + 12 + references.len() * REFERENCE_BYTES) as u32).to_be_bytes());
        bytes.extend(b"sidx");
        bytes.extend([0; 4]);
        bytes.extend(1u32.to_be_bytes());
        bytes.extend(timescale.to_be_bytes());
        bytes.extend([0; 8]);
        bytes.extend([0, 0]);
        bytes.extend((references.len() as u16).to_be_bytes());
        for (size, duration) in references {
            bytes.extend(size.to_be_bytes());
            bytes.extend(duration.to_be_bytes());
            bytes.extend(0x9000_0000u32.to_be_bytes());
        }
        bytes
    }

    #[test]
    fn lists_segments_from_the_end_of_the_index() {
        let found = segments(&index_box(24000, &[(1000, 96096), (500, 48048)]), ByteRange { first: 812, last: 899 }).unwrap();
        assert_eq!(found, [Segment { offset: 900, bytes: 1000, seconds: 4.004 }, Segment { offset: 1900, bytes: 500, seconds: 2.002 }]);
    }

    #[test]
    fn rejects_bytes_that_are_not_an_index() {
        let anywhere = ByteRange { first: 0, last: 10 };
        assert!(segments(b"\0\0\0\x10moofxxxxxxxxxxxxxxxxxxxxxxxx", anywhere).is_err());
        assert!(segments(&index_box(24000, &[(1000, 96096)])[..30], anywhere).is_err());
    }
}
