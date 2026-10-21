//! Validation-only PIV v5 scanner. ABI 1 matches the existing ctypes bridge.
//! Parsing uses checked slices; only the C boundary dereferences raw pointers.
use std::{mem, panic, slice};

const NONE: u32 = u32::MAX;
#[repr(C)]
pub struct ScanResult {
    end_offset: u64,
    error_offset: u64,
    pose_bytes: u64,
    frame_count: u32,
    error_frame: u32,
    error_object: u32,
    error_code: u32,
}
#[derive(Debug)]
struct Failure(u32, usize);
type Result<T> = std::result::Result<T, Failure>;
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(count).ok_or(Failure(1, self.pos))?;
        let value = self.data.get(self.pos..end).ok_or(Failure(1, self.pos))?;
        self.pos = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn finite(&mut self, count: usize, width: usize) -> Result<()> {
        let start = self.pos;
        let bytes = self.take(count * width)?;
        for (index, p) in bytes.chunks_exact(width).enumerate() {
            let invalid = if width == 8 {
                p[7] & 0x7f == 0x7f && p[6] & 0xf0 == 0xf0
            } else {
                p[3] & 0x7f == 0x7f && p[2] & 0x80 == 0x80
            };
            if invalid {
                self.pos = start;
                return Err(Failure(6, start + index * width));
            }
        }
        Ok(())
    }
}
fn grow<T: Clone>(v: &mut Vec<T>, count: usize, value: T) -> Result<()> {
    if count > v.len() {
        v.try_reserve(count - v.len()).map_err(|_| Failure(15, 0))?;
    }
    v.resize(count, value);
    Ok(())
}
struct Definition {
    count: usize,
    media: bool,
    handles: Vec<u16>,
}
fn definitions(data: &[u8]) -> Result<(Vec<Definition>, Vec<usize>)> {
    let mut reader = Reader { data, pos: 0 };
    let count = reader.u32()? as usize;
    if count > 65535 {
        return Err(Failure(13, 0));
    }
    let mut types = Vec::new();
    types.try_reserve(count).map_err(|_| Failure(15, 0))?;
    let mut by_id = Vec::new();
    grow(&mut by_id, 65536, usize::MAX)?;
    for _ in 0..count {
        let id = reader.u16()? as usize;
        let count = reader.u16()? as usize;
        let media = reader.u8()?;
        let reserved = reader.u8()?;
        let handle_count = reader.u16()? as usize;
        if id == 0
            || by_id[id] != usize::MAX
            || count == 0
            || count > 30000
            || media > 1
            || reserved != 0
            || handle_count != count + 1
        {
            return Err(Failure(13, 0));
        }
        let mut handles = Vec::new();
        grow(&mut handles, handle_count, 0)?;
        for handle in &mut handles {
            *handle = reader.u16()?;
        }
        handles.sort_unstable();
        if handles[0] != 0 || handles.windows(2).any(|w| w[0] == w[1]) {
            return Err(Failure(13, 0));
        }
        by_id[id] = types.len();
        types.push(Definition {
            count,
            media: media != 0,
            handles,
        });
    }
    if reader.pos != data.len() {
        return Err(Failure(13, 0));
    }
    Ok((types, by_id))
}
fn reserve_pose(result: &mut ScanResult, bytes: u64, budget: u64, pos: usize) -> Result<()> {
    let next = result
        .pose_bytes
        .checked_add(bytes)
        .ok_or(Failure(3, pos))?;
    if next > budget {
        return Err(Failure(3, pos));
    }
    result.pose_bytes = next;
    Ok(())
}
fn scan(
    reader: &mut Reader<'_>,
    metadata: &[u8],
    backgrounds: u32,
    budget: u64,
    durations: &mut [u16],
    result: &mut ScanResult,
) -> Result<()> {
    if !(1..=65535).contains(&backgrounds) {
        return Err(Failure(13, reader.pos));
    }
    let (types, by_id) = definitions(metadata)
        .map_err(|error| Failure(if error.0 == 15 { 15 } else { 13 }, reader.pos))?;
    let frames = reader.u32()?;
    result.frame_count = frames;
    if frames > 1_000_000 {
        return Err(Failure(2, reader.pos - 4));
    }
    reserve_pose(result, u64::from(frames) * 512, budget, reader.pos)?;
    if frames as usize > durations.len() {
        return Err(Failure(14, reader.pos));
    }
    let (mut tids, mut slots, mut handles, mut ids, mut checked) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (fi, output_duration) in durations.iter_mut().take(frames as usize).enumerate() {
        result.error_frame = fi as u32;
        result.error_object = NONE;
        let duration = reader.u16()?;
        let mix = reader.u8()?;
        let background = reader.u16()?;
        let background_b = if mix != 0 { reader.u16()? } else { 0 };
        if u32::from(background) >= backgrounds
            || (mix != 0 && u32::from(background_b) >= backgrounds)
        {
            return Err(Failure(4, reader.pos));
        }
        let objects = reader.u16()? as usize;
        reserve_pose(result, objects as u64 * 512, budget, reader.pos)?;
        grow(&mut tids, objects, 0usize)?;
        grow(&mut slots, objects, -1i32)?;
        slots.fill(-1);
        grow(&mut handles, objects, 0u16)?;
        grow(&mut ids, objects, 0u32)?;
        for oi in 0..objects {
            result.error_object = oi as u32;
            ids[oi] = reader.u32()?;
            let tid = reader.u16()? as usize;
            let type_index = by_id[tid];
            if type_index == usize::MAX {
                return Err(Failure(5, reader.pos));
            }
            let definition = &types[type_index];
            tids[oi] = type_index;
            reader.finite(1, 4)?;
            reader.take(5)?;
            if definition.media {
                reader.u8()?;
            }
            reserve_pose(result, definition.count as u64 * 24, budget, reader.pos)?;
            reader.finite(definition.count, 8)?;
            reader.finite(2, 4)?;
            if reader.u16()? != 0 {
                reserve_pose(result, definition.count as u64 * 24, budget, reader.pos)?;
                reader.finite(definition.count, 4)?;
            }
            if reader.u16()? != 0 {
                reserve_pose(result, definition.count as u64 * 24, budget, reader.pos)?;
                reader.finite(definition.count, 8)?;
            }
            if reader.u8()? != 0 {
                reserve_pose(result, 64, budget, reader.pos)?;
                let slot = reader.u16()? as usize;
                let handle = reader.u16()?;
                if slot >= objects {
                    return Err(Failure(9, reader.pos));
                }
                slots[oi] = slot as i32;
                handles[oi] = handle;
            }
        }
        // A reusable contiguous buffer avoids per-object hash-node allocations.
        // The wrapper revalidates duplicate IDs with Python's recovery policy.
        ids.sort_unstable();
        if ids.windows(2).any(|w| w[0] == w[1]) {
            return Err(Failure(7, reader.pos));
        }
        for oi in 0..objects {
            result.error_object = oi as u32;
            if slots[oi] >= 0
                && types[tids[slots[oi] as usize]]
                    .handles
                    .binary_search(&handles[oi])
                    .is_err()
            {
                return Err(Failure(10, reader.pos));
            }
        }
        grow(&mut checked, objects, 0u8)?;
        checked.fill(0);
        for oi in 0..objects {
            result.error_object = oi as u32;
            let mut cursor = oi as i32;
            while cursor >= 0 && checked[cursor as usize] == 0 {
                checked[cursor as usize] = 1;
                cursor = slots[cursor as usize];
            }
            if cursor >= 0 && checked[cursor as usize] == 1 {
                return Err(Failure(11, reader.pos));
            }
            cursor = oi as i32;
            while cursor >= 0 && checked[cursor as usize] == 1 {
                checked[cursor as usize] = 2;
                cursor = slots[cursor as usize];
            }
        }
        checked.fill(0);
        result.error_object = NONE;
        for _ in 0..objects {
            let slot = reader.u16()? as usize;
            if slot >= objects || checked[slot] != 0 {
                return Err(Failure(12, reader.pos));
            }
            checked[slot] = 1;
        }
        if reader.u8()? == 0 {
            reader.finite(2, 4)?;
            reader.finite(1, 8)?;
            reader.finite(1, 4)?;
        }
        *output_duration = duration;
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn styk_piv_scan_abi() -> u32 {
    1
}
#[no_mangle]
pub extern "C" fn styk_piv_scan_implementation() -> u32 {
    2
}
#[no_mangle]
pub extern "C" fn styk_piv_scan_result_size() -> u32 {
    mem::size_of::<ScanResult>() as u32
}

/// # Safety
/// Callers supply valid, nonoverlapping buffers of the stated lengths, with
/// aligned writable output/durations. Input buffers stay immutable until return.
/// No allocation, pointer or Rust-owned object escapes to the caller.
#[no_mangle]
pub unsafe extern "C" fn styk_piv_scan(
    data: *const u8,
    size: u64,
    start: u64,
    metadata: *const u8,
    metadata_size: u64,
    backgrounds: u32,
    budget: u64,
    durations: *mut u16,
    capacity: u32,
    output: *mut ScanResult,
) -> u32 {
    if output.is_null() {
        return 16;
    }
    let result = &mut *output;
    *result = ScanResult {
        end_offset: start,
        error_offset: 0,
        pose_bytes: 0,
        frame_count: 0,
        error_frame: NONE,
        error_object: NONE,
        error_code: 0,
    };
    if data.is_null() || size > 256 * 1024 * 1024 || start > size {
        result.error_code = 1;
        result.error_offset = start;
        return 1;
    }
    if metadata.is_null() || metadata_size > 64 * 1024 * 1024 {
        result.error_code = 13;
        result.error_offset = start;
        return 13;
    }
    let bytes = slice::from_raw_parts(data, size as usize);
    let metadata = slice::from_raw_parts(metadata, metadata_size as usize);
    let durations = if durations.is_null() || capacity == 0 {
        &mut []
    } else {
        slice::from_raw_parts_mut(durations, capacity.min(1_000_000) as usize)
    };
    let mut reader = Reader {
        data: bytes,
        pos: start as usize,
    };
    let outcome = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        scan(
            &mut reader,
            metadata,
            backgrounds,
            budget,
            durations,
            result,
        )
    }));
    let failure = match outcome {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error),
        Err(_) => Some(Failure(16, reader.pos)),
    };
    if let Some(Failure(code, offset)) = failure {
        result.error_code = code;
        result.error_offset = offset as u64;
    }
    result.end_offset = reader.pos as u64;
    result.error_code
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn abi_and_checked_reads() {
        assert_eq!(mem::size_of::<ScanResult>(), 40);
        let mut reader = Reader {
            data: &[0, 1],
            pos: 0,
        };
        assert_eq!(reader.u16().unwrap(), 256);
        assert!(reader.u8().is_err());
        assert_eq!(reader.pos, 2);
    }
    #[test]
    fn exponent_validation_retains_signed_zero_and_subnormals() {
        for value in [0.0f64, -0.0, f64::MIN_POSITIVE, f64::from_bits(1), f64::MAX] {
            assert!(Reader {
                data: &value.to_le_bytes(),
                pos: 0
            }
            .finite(1, 8)
            .is_ok());
        }
        for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            assert!(Reader {
                data: &value.to_le_bytes(),
                pos: 0
            }
            .finite(1, 8)
            .is_err());
        }
    }
}
