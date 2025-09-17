pub mod ws;
pub mod udp;
pub mod http;

use hyper::body::Bytes;
use std::time::Duration;

pub const MTU: usize = 1500;
const TIMEOUT: Duration = Duration::from_millis(50);

pub struct Headers {
    pub headers: Option<Bytes>,
}

fn prepare_headers(buf: &[Bytes]) -> Bytes {
  Bytes::copy_from_slice(&[&buf[0][..], &buf[1][..]].concat())
}

fn validate_bos_and_tags(data: & Bytes) -> core::result::Result<&Bytes, ()> {
  let n_segs = data[26] as usize;
  let offset = 27+n_segs;
  if data.len() < 27 + 8 { return Err(()) }
  if matches!(&data[offset..offset+8], b"OpusTags" | b"OpusHead") {
    return Ok(data);
  }
  Err(())
}
