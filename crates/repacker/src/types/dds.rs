use crate::types::rfi::{RFI, S3TC};
use std::mem::size_of;

#[repr(C, packed)]
pub struct Header {
	pub size: u32,
	pub flags: u32,
	pub height: u32,
	pub width: u32,
	pub pitch_or_linear_size: u32,
	pub depth: u32,
	pub mip_map_count: u32,
	pub reserved1: [u32; 11],
	pub ddspf: PixelFormat,
	pub caps: u32,
	pub caps2: u32,
	pub caps3: u32,
	pub caps4: u32,
	pub reserved2: u32,
}

impl Header {
	pub fn new(rfi: &RFI) -> Result<Self, Box<dyn std::error::Error>> {
		let pf = PixelFormat::new(rfi)?;
		let header = Header {
			size: size_of::<Header>() as u32,
			flags: 0,
			height: rfi.header.height,
			width: rfi.header.width,
			pitch_or_linear_size: 0,
			depth: 0,
			mip_map_count: 0,
			reserved1: vec![0; 11].as_slice().try_into()?,
			ddspf: pf,
			caps: 0,
			caps2: 0,
			caps3: 0,
			caps4: 0,
			reserved2: 0,
		};

		Ok(header)
	}
}

#[repr(C, packed)]
pub struct PixelFormat {
	pub size: u32,
	pub flags: u32,
	pub fourcc: [u8; 4],
	pub rgb_bit_count: u32,
	pub r_bit_mask: u32,
	pub g_bit_mask: u32,
	pub b_bit_mask: u32,
	pub a_bit_mask: u32,
}

impl PixelFormat {
	pub fn new(rfi: &RFI) -> Result<Self, Box<dyn std::error::Error>> {
		let s3tc = S3TC::try_from(rfi.header.s3tc)?;
		let fourcc = match s3tc {
			S3TC::DXT1(_) => b"DXT1",
			S3TC::DXT5(_) => b"DXT5",
			S3TC::BC4U(_) => b"BC4U",
			S3TC::BC5U(_) => b"BC5U",
			S3TC::RGB(_) => b"\0\0\0\0",
		};

		let pf = match s3tc {
			S3TC::RGB(_) => PixelFormat {
				size: size_of::<PixelFormat>() as u32,
				flags: 0x00000040,
				fourcc: fourcc.to_owned(),
				rgb_bit_count: 24,
				r_bit_mask: 0xFF0000,
				g_bit_mask: 0x00FF00,
				b_bit_mask: 0x0000FF,
				a_bit_mask: 0,
			},
			_ => PixelFormat {
				size: size_of::<PixelFormat>() as u32,
				flags: 0x00000004,
				fourcc: fourcc.to_owned(),
				rgb_bit_count: 0,
				r_bit_mask: 0,
				g_bit_mask: 0,
				b_bit_mask: 0,
				a_bit_mask: 0,
			},
		};

		Ok(pf)
	}
}
