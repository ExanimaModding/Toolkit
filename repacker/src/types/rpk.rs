use crate::{
    metadata::{MagicBytes, Metadata},
    types::ex_str::ExanimaString,
    utils::{any_as_u8_slice, green, is_file_valid, red, yellow},
};
use bitstream_io::{
    read::{BitRead, BitReader},
    write::{BitWrite, BitWriter},
    LittleEndian,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, read, write, File},
    io::SeekFrom,
    mem,
    path::PathBuf,
};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TableEntry {
    pub name: ExanimaString,
    pub offset: u32,
    pub size: u32,
    pub padding: [u32; 2],
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RPK {
    pub filetype: String,
    pub use_file_extensions: bool,
}

impl RPK {
    pub fn get_name(entry_name: ExanimaString, pos: usize) -> String {
        match String::try_from(entry_name) {
            Ok(name) => name,
            Err(e) => {
                // panic for now, but later can be changed
                // to returning an error if absolutely necessary
                panic!(
                    "❗ Could not unpack at table entry position ({}) {}",
                    pos, e
                );
            }
        }
    }

    pub fn pack(src: &str, dest: &str) -> Result<(), std::io::Error> {
        let src_path = PathBuf::from(src);
        let src_name = src_path.file_name().unwrap();
        let src_name_str = src_name.to_str().unwrap();
        let mut dest_path = PathBuf::from(dest);

        create_dir_all(&dest_path)?;

        let mut meta_path = src_path.clone();
        meta_path.push("metadata.toml");
        let metadata = Metadata::<RPK>::from(meta_path.to_str().unwrap());
        if metadata.is_err() {
            eprintln!("❗ No metadata file found in '{}'", red(src_name_str));
            return Err(metadata.err().unwrap());
        };
        let metadata = metadata?;

        dest_path.push(src_name);
        dest_path.set_extension(&metadata.0.filetype);

        let magic = MagicBytes::try_from(metadata.0.filetype.as_str()).unwrap();
        if magic != MagicBytes::RPK {
            panic!("❗ Folder, '{}', is not an RPK format", red(src_name_str))
        }

        let mut writer = BitWriter::endian(File::create(dest_path)?, LittleEndian);
        writer.write(32, magic as u32)?;

        let mut table_length: u32 = 0;

        // Filter out non-game files
        let mut paths: Vec<PathBuf> = src_path
            .read_dir()
            .unwrap()
            .filter_map(|r| {
                let entry = r.as_ref().unwrap();
                if !is_file_valid(&entry) {
                    let mut reader = bitstream_io::BitReader::endian(
                        std::fs::File::open(&entry.path()).unwrap(),
                        bitstream_io::LittleEndian,
                    );
                    let invalid_magic = match reader.read::<u32>(32) {
                        Ok(magic) => magic,
                        Err(_) => 0,
                    };

                    if entry.file_name().to_str().unwrap() != "metadata.toml" {
                        eprintln!(
                            "⚠️ Ignoring file, '{}' ({:#08X}), in '{}'",
                            yellow(entry.file_name().to_str().unwrap()),
                            invalid_magic,
                            green(src_name_str)
                        );
                    }
                    return None;
                }
                table_length += 1;

                Some(r.unwrap().path())
            })
            .collect();

        // Sort files to match original order
        paths.sort_by_key(|dir| {
            if metadata.0.use_file_extensions {
                dir.to_owned()
            } else {
                dir.with_extension("")
            }
        });

        // Pack nested folders
        // for entry in &paths {
        //     if !entry.path().is_dir() {
        //         continue;
        //     }
        //     // check for metadata
        //     // if metadata, pack the folder into .packed folder
        // }

        let table_size_bytes = table_length * 32;
        unsafe {
            let bytes = any_as_u8_slice(&table_size_bytes);
            writer.write_bytes(bytes)?;
        }

        // Populate table entries
        let mut offset: u32 = 0;
        for path in &paths {
            // Corrupts data if there are a mix of files with and without extensions.
            // Metadata should except file names without extensions if
            // metadata.0.use_file_extensions is true.
            let name = if !metadata.0.use_file_extensions {
                String::from(path.file_stem().unwrap().to_str().unwrap())
            } else {
                String::from(path.file_name().unwrap().to_str().unwrap())
            };

            let ex_name = ExanimaString::try_from(name.clone());
            if ex_name.is_err() {
                panic!(
                    "❗ Make file name, specifically '{}', in {} 16 characters or less",
                    red(name.as_str()),
                    green(src_name_str)
                )
            }
            let ex_name = ex_name.unwrap();

            let bytes = read(&path)?;
            let table_entry = TableEntry {
                name: ex_name,
                offset,
                size: bytes.len() as u32,
                padding: [0, 0],
            };

            unsafe {
                let table_u8 = any_as_u8_slice(&table_entry);
                writer.write_bytes(table_u8)?;
            }

            offset += bytes.len() as u32;
        }

        // Write raw data
        for path in &paths {
            let bytes = read(&path)?;
            writer.write_bytes(bytes.as_slice())?;
        }

        println!("✔️ {} done", green(src_name_str));

        Ok(())
    }

    async unsafe fn read_struct<T: Copy>(
        reader: &mut BitReader<File, LittleEndian>,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let vec = reader.read_to_vec(mem::size_of::<T>())?;
        let (_, body, _tail) = vec.align_to::<T>();
        Ok((&body[0]).clone())
    }

    pub async fn unpack(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let src_path = PathBuf::from(src);
        let src_name_str = src_path.file_name().unwrap().to_str().unwrap();
        let mut dest_path = PathBuf::from(dest);

        let mut reader = BitReader::endian(File::open(&src_path)?, LittleEndian);

        let magic = reader.read::<u32>(32)?;
        let magic = MagicBytes::try_from(magic)?;
        if magic != MagicBytes::RPK {
            // Since magic is valid, use corresponding file type's
            // unpack() and return instead of doing panic!()
            panic!("❗ '{}' must be an RPK format", red(src_name_str));
        }

        let table_size_bytes = reader.read::<u32>(32)?;
        let table_length = table_size_bytes / 32;
        let mut table_entries: Vec<TableEntry> = Vec::new();
        unsafe {
            for _ in 0..table_length {
                table_entries.push(RPK::read_struct::<TableEntry>(&mut reader).await?)
            }
        }

        create_dir_all(&dest_path)?;
        let data_start_pos = reader.position_in_bits()?;
        // Inaccurate if there are a mix of files with and without extensions.
        let file_ext_exists = {
            let name = RPK::get_name(table_entries.get(0).unwrap().name, 0);
            let mut dest_path = dest_path.clone();
            dest_path.push(&name);

            match dest_path.extension() {
                Some(ext) if ext.len() == 0 => false,
                Some(_) => true,
                None => false,
            }
        };

        // let mut set = JoinSet::new();
        let mut handles = vec![];
        for (i, entry) in table_entries.iter().enumerate() {
            let mut name = RPK::get_name(entry.name, i);

            let seek_to = data_start_pos + (entry.offset as u64 * 8);
            reader.seek_bits(SeekFrom::Start(seek_to))?;
            let buf = reader.read_to_vec(entry.size as usize)?;

            let dest_path = dest_path.clone();
            handles.push(tokio::spawn(async move {
                // 'stool_brass c2.' in Objlib.rpk ends with a '.'
                let mut dest_path = dest_path.clone();
                if name.ends_with(".") {
                    name.push('.');
                }
                dest_path.push(&name);

                let magic = u32::from_le_bytes(buf[0..4].try_into().unwrap());

                if !file_ext_exists {
                    match MagicBytes::try_from(magic) {
                        Ok(magic) => {
                            dest_path.set_extension(String::from(magic));
                        }
                        Err(e) => {
                            if magic != 0 {
                                eprintln!(
                                    "⚠️ Unknown file type from file, '{}' ({:#08X}), in {}: {}",
                                    yellow(&name),
                                    &magic,
                                    green(
                                        &dest_path
                                            .parent()
                                            .unwrap()
                                            .file_name()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                    ),
                                    e
                                )
                            }
                            dest_path.set_extension("unknown");
                        }
                    };
                }

                write(&dest_path, buf).unwrap();
            }));
        }
        futures::future::join_all(handles).await;

        let ext = String::from(src_path.extension().unwrap().to_str().unwrap());
        dest_path.push("metadata.toml");
        let metadata: Metadata<RPK> = Metadata(RPK {
            filetype: ext,
            use_file_extensions: file_ext_exists,
        });
        metadata.write_to(dest_path.to_str().unwrap())?;

        println!("✔️ {} done", green(src_name_str));

        Ok(())
    }
}
