use hyper::body::Bytes;

#[derive(Debug, Clone)]
pub struct OggHeaders {
  pub head: Bytes,
  pub tags: Bytes
}

impl OggHeaders {
  pub fn new(headers: (Bytes, Bytes)) -> Self {
    Self {
      head: headers.0,
      tags: headers.1,
    }
  }
}

pub enum OggHeaderType {
  Head(Bytes),
  Tags(Bytes),
  /// Represents a catch-all of other ogg page data or header fragments
  None
}

fn get_header_segment(data: &Bytes) -> Result<usize, ()> {
  let n_segs = data[26] as usize;
  let offset = 27 + n_segs;
  if data.len() < 27 + 8 { 
    return Err(()) 
  }
  Ok(offset)
}

// pub fn validate_tags(data: &Bytes) -> Result<&Bytes, ()> {
//   let offset = get_header_segment(data)?;
//   if &data[offset..offset + 8] == b"OpusTags" {
//     println!("header tags found");
//     return Ok(data);
//   }
//   Err(())
// }
//
// pub fn validate_header(data: &Bytes) -> Result<&Bytes, ()> {
//   let offset = get_header_segment(data)?;
//   if &data[offset..offset + 8] == b"OpusHead" {
//     println!("header found");
//     return Ok(data);
//   }
//   Err(())
// }

pub fn parse_ogg_headers(data: &Bytes) -> OggHeaderType {
  let Ok(offset) = get_header_segment(data) else { return OggHeaderType::None };
  if &data[offset..offset + 8] == b"OpusHead" {
    println!("header found");
    return OggHeaderType::Head(data.clone()); 
  }
  if &data[offset..offset + 8] == b"OpusTags" {
    println!("header tags found");
    return OggHeaderType::Tags(data.clone()); 
  }
  OggHeaderType::None
}


