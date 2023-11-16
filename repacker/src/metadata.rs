// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{
    de::DeserializeOwned,
    {Deserialize, Serialize},
};
use std::{
    convert::{From, TryFrom},
    fmt::{Display, Formatter},
    fs::File,
    io::{BufReader, Read, Result, Write},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Metadata<T>(pub T);

impl<T> Metadata<T>
where
    T: DeserializeOwned + Serialize,
{
    pub fn from(path: &str) -> Result<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        Ok(Self(toml::from_str(contents.as_str()).unwrap()))
    }

    pub fn write_to(&self, path: &str) -> Result<()> {
        let mut f = File::create(path)?;
        let toml = toml::to_string(&self.0).unwrap();

        write!(f, "{}", toml)
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MagicBytes {
    Factory0 = 0xAFCE0F00,
    Factory1 = 0xAFCE0F01,
    FTB = 0x3EEFBD01,
    PWR = 0xAFCE01CE,
    RCR = 0x00CDAC06,
    RFC1 = 0x3D23AFCF,
    RFC2 = 0x3D21AFCF,
    RFI = 0x1D2D3DC6,
    RFP = 0xAFDFBD10,
    RFT = 0x3EEFAD01,
    RPK = 0xAFBF0C01,
    RSG = 0xDA7AEA02,
    RSQ = 0x3D000000,
    WAV = 0x46464952,
    RUI = 0x615B0A0D,
    RAB = 0x7EF6DC8A,
    RPP = 0xDCEACCD2,
    DET = 0xDCD2EC40,
    IDK1 = 0x7EF6D298,
    IDK2 = 0x3F49C9CA,
    IDK3 = 0xC2D8DEF2,
    IDK4 = 0xCAEC68C6,
    IDK5 = 0x7EF6D0A6,
    IDK6 = 0xD2C8CCCC,
    IDK7 = 0xCAEC66C6,
}

impl Display for MagicBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#08X}", *self as u32)
    }
}

impl TryFrom<u32> for MagicBytes {
    type Error = &'static str;

    fn try_from(n: u32) -> std::result::Result<Self, Self::Error> {
        match n {
            0xAFCE0F00 => Ok(MagicBytes::Factory0),
            0xAFCE0F01 => Ok(MagicBytes::Factory1),
            0x3EEFBD01 => Ok(MagicBytes::FTB),
            0xAFCE01CE => Ok(MagicBytes::PWR),
            0x00CDAC06 => Ok(MagicBytes::RCR),
            0x3D23AFCF => Ok(MagicBytes::RFC1),
            0x3D21AFCF => Ok(MagicBytes::RFC2),
            0x1D2D3DC6 => Ok(MagicBytes::RFI),
            0xAFDFBD10 => Ok(MagicBytes::RFP),
            0x3EEFAD01 => Ok(MagicBytes::RFT),
            0xAFBF0C01 => Ok(MagicBytes::RPK),
            0xDA7AEA02 => Ok(MagicBytes::RSG),
            0x3D000000 => Ok(MagicBytes::RSQ),
            0x46464952 => Ok(MagicBytes::WAV),
            0x615B0A0D => Ok(MagicBytes::RUI),
            0x7EF6DC8A => Ok(MagicBytes::RAB),
            0xDCEACCD2 => Ok(MagicBytes::RPP),
            0xDCD2EC40 => Ok(MagicBytes::DET),
            0x7EF6D298 => Ok(MagicBytes::IDK1),
            0x3F49C9CA => Ok(MagicBytes::IDK2),
            0xC2D8DEF2 => Ok(MagicBytes::IDK3),
            0xCAEC68C6 => Ok(MagicBytes::IDK4),
            0x7EF6D0A6 => Ok(MagicBytes::IDK5),
            0xD2C8CCCC => Ok(MagicBytes::IDK6),
            0xCAEC66C6 => Ok(MagicBytes::IDK7),
            _ => Err("Cannot get magic from number"),
        }
    }
}

impl TryFrom<&str> for MagicBytes {
    type Error = &'static str;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            // 0xAFCE0F00 => Ok(MagicBytes::Factory0),
            "fty" => Ok(MagicBytes::Factory1),
            "ftb" => Ok(MagicBytes::FTB),
            "pwr" => Ok(MagicBytes::PWR),
            "rcr" => Ok(MagicBytes::RCR),
            "rfc" => Ok(MagicBytes::RFC1),
            // 0x3D21AFCF => Ok(MagicBytes::RFC2),
            "rfi" => Ok(MagicBytes::RFI),
            "rfp" => Ok(MagicBytes::RFP),
            "rft" => Ok(MagicBytes::RFT),
            "fds" | "flb" | "rml" | "rpk" => Ok(MagicBytes::RPK),
            "rcp" | "rsg" => Ok(MagicBytes::RSG),
            "rsq" => Ok(MagicBytes::RSQ),
            "wav" => Ok(MagicBytes::WAV),
            "rui" => Ok(MagicBytes::RUI),
            "rab" => Ok(MagicBytes::RAB),
            "rpp" => Ok(MagicBytes::RPP),
            "det" => Ok(MagicBytes::DET),
            "idk1" => Ok(MagicBytes::IDK1),
            "idk2" => Ok(MagicBytes::IDK2),
            "idk3" => Ok(MagicBytes::IDK3),
            "idk4" => Ok(MagicBytes::IDK4),
            "idk5" => Ok(MagicBytes::IDK5),
            "idk6" => Ok(MagicBytes::IDK6),
            "idk7" => Ok(MagicBytes::IDK7),
            _ => Err("Cannot get magic from string"),
        }
    }
}

impl From<MagicBytes> for String {
    fn from(b: MagicBytes) -> String {
        match b {
            MagicBytes::Factory0 | MagicBytes::Factory1 => String::from("fty"),
            MagicBytes::FTB => String::from("ftb"),
            MagicBytes::PWR => String::from("pwr"),
            MagicBytes::RCR => String::from("rcr"),
            MagicBytes::RFC1 | MagicBytes::RFC2 => String::from("rfc"),
            MagicBytes::RFI => String::from("rfi"),
            MagicBytes::RFP => String::from("rfp"),
            MagicBytes::RFT => String::from("rft"),
            MagicBytes::RPK => String::from("rpk"),
            MagicBytes::RSG => String::from("rsg"),
            MagicBytes::RSQ => String::from("rsq"),
            MagicBytes::WAV => String::from("wav"),
            MagicBytes::RUI => String::from("rui"),
            MagicBytes::RAB => String::from("rab"),
            MagicBytes::RPP => String::from("rpp"),
            MagicBytes::DET => String::from("det"),
            MagicBytes::IDK1 => String::from("idk1"),
            MagicBytes::IDK2 => String::from("idk2"),
            MagicBytes::IDK3 => String::from("idk3"),
            MagicBytes::IDK4 => String::from("idk4"),
            MagicBytes::IDK5 => String::from("idk5"),
            MagicBytes::IDK6 => String::from("idk6"),
            MagicBytes::IDK7 => String::from("idk7"),
        }
    }
}
