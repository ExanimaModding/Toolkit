use crate::metadata::{MagicBytes, Metadata};
use crate::types::ex_str::ExanimaString;
use crate::utils::{any_as_u8_slice, is_file_valid};
use bitstream_io::{BitRead, BitReader, BitWrite, BitWriter, LittleEndian};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read, write, File};
use std::io::SeekFrom;
use std::mem;
use std::path::PathBuf;

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
    pub fn pack(src: &str, dest: &str) -> Result<(), std::io::Error> {
        let src_path = PathBuf::from(src);
        let mut dest_path = PathBuf::from(dest);

        create_dir_all(&dest_path)?;

        let mut meta_path = src_path.clone();
        meta_path.push("metadata.toml");
        // dbg!("pack: metadata '{}'", &meta_path.to_str().unwrap());
        let metadata = Metadata::<RPK>::from(meta_path.to_str().unwrap());
        if metadata.is_err() {
            eprintln!(
                "No metadata file found in '{}'",
                &src_path.to_str().unwrap()
            );
            return Err(metadata.err().unwrap());
        };
        let metadata = metadata.unwrap();

        dest_path.push(src_path.file_name().unwrap());
        dest_path.set_extension(&metadata.0.filetype);

        let magic = MagicBytes::try_from(metadata.0.filetype.as_str()).unwrap();
        if magic != MagicBytes::RPK {
            panic!(
                "Folder is not an RPK format at '{}'",
                &src_path.to_str().unwrap()
            )
        }

        let mut writer = BitWriter::endian(File::create(dest_path)?, LittleEndian);
        writer.write(32, magic as u32)?;

        // Sort files to match original order
        let mut table_length: u32 = 0;
        let mut paths: Vec<PathBuf> = src_path
            .read_dir()
            .unwrap()
            .filter_map(|r| {
                let entry = r.as_ref().unwrap();
                if !is_file_valid(&entry) {
                    let mut reader =
                        BitReader::endian(File::open(&entry.path()).unwrap(), LittleEndian);
                    let invalid_magic = reader.read::<u32>(32).unwrap();
                    eprintln!(
                        "Ignoring file '{}' ({:#08X}) at '{}'",
                        entry.file_name().to_str().unwrap(),
                        invalid_magic,
                        entry.path().to_str().unwrap(),
                    );
                    return None;
                }
                table_length += 1;

                Some(r.unwrap().path())
            })
            .collect();

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

        let sty = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-");

        // Populate table entries
        let pb_table = ProgressBar::new(table_length as u64);
        pb_table.set_style(sty.clone());
        let mut offset: u32 = 0;
        for path in &paths {
            pb_table.set_message(format!(
                "{}",
                &path.file_name().unwrap().to_str().unwrap().yellow()
            ));

            // Corrupts data if there are a mix of files with and without extensions.
            // Metadata should except file names without extensions if
            // metadata.0.use_file_extensions is true.
            let name = if !metadata.0.use_file_extensions {
                String::from(path.file_stem().unwrap().to_str().unwrap())
            } else {
                String::from(path.file_name().unwrap().to_str().unwrap())
            };

            let ex_name = ExanimaString::try_from(name.clone()).expect(
                format!(
                    "Make file name, specifically '{}', 16 characters or less",
                    name
                )
                .as_str(),
            );

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

            pb_table.inc(1);
        }
        pb_table.finish_with_message(format!(
            "{} table done",
            &src_path.file_name().unwrap().to_str().unwrap().green(),
        ));

        // Write raw data
        let pb_data = ProgressBar::new(table_length as u64);
        pb_data.set_style(sty.clone());
        for path in &paths {
            pb_data.set_message(format!(
                "{}",
                &path.file_name().unwrap().to_str().unwrap().yellow()
            ));

            let bytes = read(&path)?;
            writer.write_bytes(bytes.as_slice())?;

            pb_data.inc(1);
        }
        pb_data.finish_with_message(format!(
            "{} data done",
            &src_path.file_name().unwrap().to_str().unwrap().green(),
        ));

        Ok(())
    }

    unsafe fn read_struct<T: Copy>(
        reader: &mut BitReader<File, LittleEndian>,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let vec = reader.read_to_vec(mem::size_of::<T>())?;
        let (_, body, _tail) = vec.align_to::<T>();
        Ok((&body[0]).clone())
    }

    pub fn unpack(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let src_path = PathBuf::from(src);
        let mut dest_path = PathBuf::from(dest);

        let mut reader = BitReader::endian(File::open(&src_path)?, LittleEndian);

        let magic = reader.read::<u32>(32)?;
        let magic = MagicBytes::try_from(magic)?;
        if magic != MagicBytes::RPK {
            // Since magic is valid, use corresponding file type's
            // unpack() and return instead of doing panic!()
            panic!("File type must be an RPK format");
        }

        create_dir_all(&dest_path)?;

        let table_size_bytes = reader.read::<u32>(32)?;
        let table_length = table_size_bytes / 32;

        let mut table_entries: Vec<TableEntry> = Vec::new();
        unsafe {
            for _ in 0..table_length {
                table_entries.push(RPK::read_struct::<TableEntry>(&mut reader)?)
            }
        }

        let data_start_pos = reader.position_in_bits()?;

        let sty = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-");

        let pb_table = ProgressBar::new(table_entries.len() as u64);
        pb_table.set_style(sty.clone());

        let mut file_ext_exists = false;
        for (i, entry) in table_entries.iter().enumerate() {
            let mut name = match String::try_from(entry.name) {
                Ok(name) => name,
                Err(e) => {
                    eprintln!("Could not unpack at table entry position ({}) {}", i, e);
                    continue;
                }
            };

            pb_table.set_message(format!("{}", &name.yellow()));

            let seek_to = data_start_pos + (entry.offset as u64 * 8);
            reader.seek_bits(SeekFrom::Start(seek_to))?;
            let buf = reader.read_to_vec(entry.size as usize)?;

            // 'stool_brass c2.' in Objlib.rpk ends with a '.'
            let mut dest_path = dest_path.clone();
            if name.ends_with(".") {
                name.push('.');
            }
            dest_path.push(&name);
            let has_ext = match dest_path.extension() {
                Some(ext) if ext.len() == 0 => false,
                Some(_) => true,
                None => false,
            };

            let magic = u32::from_le_bytes(buf[0..4].try_into()?);

            if !has_ext {
                match MagicBytes::try_from(magic) {
                    Ok(magic) => {
                        dest_path.set_extension(String::from(magic));
                    }
                    Err(e) => {
                        if magic != 0 {
                            eprintln!(
                                "Unknown file type from file '{}' ({:#08X}) at {}: {}",
                                &name,
                                &magic,
                                &dest_path.to_str().unwrap(),
                                e
                            );
                        }
                        dest_path.set_extension("unknown");
                    }
                };
            } else {
                file_ext_exists = true;
            }

            write(&dest_path, buf)?;

            pb_table.inc(1);
        }
        pb_table.finish_with_message(format!(
            "{} done",
            &src_path.file_name().unwrap().to_str().unwrap().green(),
        ));

        let ext = String::from(src_path.extension().unwrap().to_str().unwrap());
        dest_path.push("metadata.toml");
        let metadata: Metadata<RPK> = Metadata(RPK {
            filetype: ext,
            use_file_extensions: file_ext_exists,
        });
        metadata.write_to(dest_path.to_str().unwrap())?;

        Ok(())
    }
}
