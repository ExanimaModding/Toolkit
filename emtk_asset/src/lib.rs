use std::{
	borrow::Cow,
	cmp::Ordering,
	fmt,
	fs::File,
	hash::{Hash, Hasher},
	io::{self, Read, Seek, SeekFrom, Write},
	mem,
	path::{Component, Path, PathBuf},
};

pub use deku;
use deku::{
	DekuError, DekuRead, DekuReader, DekuWrite,
	reader::{Reader, ReaderRet},
	writer::Writer,
};

pub use entry::{Entry, EntryError};
pub mod entry {
	use super::*;

	#[derive(Debug, thiserror::Error)]
	pub enum EntryError<'a> {
		#[error(transparent)]
		Deku(#[from] DekuError),
		#[error("Entry with name not found: {name}")]
		NotFound { name: Cow<'a, str> },
		#[error("Expected {expected}, found {found}")]
		InvalidPath {
			expected: Cow<'a, str>,
			found: Cow<'a, Path>,
		},
		#[error("Invalid Utf8: {0}")]
		InvalidUtf8(Cow<'a, str>),
		#[error(transparent)]
		Io(#[from] io::Error),
	}

	#[derive(Debug, Eq, DekuRead, DekuWrite)]
	pub struct Entry {
		#[deku(
			reader = "Self::read_name(deku::reader).map(Box::from)",
			writer = "Self::write_name(deku::writer, &name)"
		)]
		pub name: Box<str>,
		pub byte_offset: u32,
		#[deku(pad_bytes_after = "8")]
		pub byte_length: u32,
	}

	impl Entry {
		/// The amount of bytes the name of the entry takes up.
		pub const NAME_LENGTH: usize = 16;

		/// The amount of bytes a single entry takes up on disk.
		pub const RAW_SIZE: usize = 32;

		#[inline]
		pub fn new(name: &str) -> Self {
			Self {
				name: name.into(),
				byte_offset: Default::default(),
				byte_length: Default::default(),
			}
		}

		#[inline]
		pub fn with_byte_length(mut self, byte_length: u32) -> Self {
			self.byte_length = byte_length;
			self
		}

		/// Takes in a reader and reads bytes at the current streamer position until
		/// reaching the [`Entry`]'s byte_length.
		#[inline]
		pub fn read_bytes<R: Read + Seek>(
			&self,
			reader: &mut Reader<R>,
		) -> Result<Vec<u8>, DekuError> {
			let mut bytes = Vec::with_capacity(self.byte_length as _);

			// SAFETY:
			// 1. [capacity()] is used to set the `new_len`
			// 2. Elements are initialized immediately after [set_len()]
			//
			// If the reader fails, the vector will contain uninitialized elements. In this
			// case, we propogate the error and the vector is dropped immediately.
			unsafe {
				bytes.set_len(bytes.capacity());
				reader.read_bytes(bytes.capacity(), &mut bytes, Default::default())?;
			}

			Ok(bytes)
		}

		/// Helper for returning the offset of the entry's asset relative to the start
		/// of a package.
		#[inline]
		pub fn asset_offset(&self, table_byte_length: u32) -> u32 {
			// The 4 + 4 is the package's magic byte plus the `table_byte_length` byte
			// itself, respectively. These bytes are found at the start of a package.
			4 + 4 + table_byte_length + self.byte_offset
		}

		#[inline]
		fn read_name<R: Read + Seek>(reader: &mut Reader<R>) -> Result<String, DekuError> {
			let mut bytes = [0u8; Self::NAME_LENGTH];
			match reader.read_bytes(Self::NAME_LENGTH, &mut bytes, Default::default()) {
				Ok(ReaderRet::Bytes) => {
					let mut name = String::new();
					for byte in bytes {
						match byte {
							0 => break,
							b => name.push(b as char),
						}
					}
					Ok(name)
				}
				Ok(ReaderRet::Bits(_)) => {
					Err(DekuError::InvalidParam("Bytes are not aligned".into()))
				}
				Err(e) => Err(e),
			}
		}

		#[inline]
		fn write_name<W: Write + Seek>(
			writer: &mut Writer<W>,
			name: &str,
		) -> Result<(), DekuError> {
			let mut buf = [0; 16];
			for (i, c) in name.chars().enumerate() {
				if i >= 16 {
					break;
				}
				buf[i] = c as u8;
			}
			writer.write_bytes(&buf)
		}
	}

	impl fmt::Display for Entry {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.write_str(&self.name)
		}
	}

	impl PartialEq for Entry {
		#[inline]
		fn eq(&self, other: &Self) -> bool {
			self.name == other.name
		}
	}

	impl PartialEq<str> for Entry {
		#[inline]
		fn eq(&self, other: &str) -> bool {
			*self.name == *other
		}
	}

	impl PartialOrd for Entry {
		#[inline]
		fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
			self.name.partial_cmp(&other.name)
		}
	}

	impl Ord for Entry {
		#[inline]
		fn cmp(&self, other: &Self) -> Ordering {
			self.name.cmp(&other.name)
		}
	}

	impl Hash for Entry {
		#[inline]
		fn hash<H: Hasher>(&self, state: &mut H) {
			self.name.hash(state);
		}
	}

	impl<'a> From<&'a Entry> for &'a str {
		#[inline]
		fn from(value: &'a Entry) -> &'a str {
			&value.name
		}
	}

	impl From<Entry> for String {
		#[inline]
		fn from(value: Entry) -> Self {
			value.name.into_string()
		}
	}
}

pub enum Handle {
	File(File),
	Directory(PathBuf),
}

pub struct Package {
	handle: Handle,
}

impl Handle {
	#[inline]
	pub fn entries(&self) -> Result<Vec<Entry>, EntryError<'_>> {
		let entries = match &self {
			Handle::File(file) => {
				let mut reader = Reader::new(file);
				let read_table = read_table(&mut reader)?;

				let mut entries = Vec::new();
				for entry in read_table {
					let entry = entry?;
					entries.push(entry);
				}

				entries
			}
			Handle::Directory(path) => {
				let read_dir = path.read_dir()?;

				let mut entries = Vec::new();
				for dir_entry in read_dir {
					let dir_entry = dir_entry?;

					let file_name = dir_entry.file_name();
					let file_name = file_name.to_str().ok_or(EntryError::InvalidUtf8(
						file_name.display().to_string().into(),
					))?;

					let entry =
						Entry::new(file_name).with_byte_length(dir_entry.metadata()?.len() as _);
					entries.push(entry);
				}

				entries
			}
		};

		Ok(entries)
	}
}

impl Package {
	pub const MAGIC: &'static [u8; 4] = b"\x01\x0C\xBF\xAF";

	#[inline]
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, EntryError<'static>> {
		let path = path.as_ref();

		let handle = if path.is_file() {
			Handle::File(File::open(path)?)
		} else if path.is_dir() {
			Handle::Directory(path.to_owned())
		} else {
			return Err(EntryError::InvalidPath {
				expected: "directory or file at path".into(),
				found: path.to_owned().into(),
			});
		};

		Ok(Self {
			handle: handle.into(),
		})
	}

	#[inline]
	pub fn entries(&self) -> Result<Vec<Entry>, EntryError<'_>> {
		self.handle.entries()
	}

	/// A naive implementation of traversing entries within a [`Package`] with basic
	/// loose file support but has a few points to consider that may result in
	/// unintended behavior:
	///
	/// - If `self.path` points to a file, `path` could be pointing to another
	/// [`Package`] which would return the bytes of that entire package,
	/// [`Package`]s can be big in size...
	/// - If `self.path` points to a directory, `path` could be pointing to another
	/// directory which would panic with a todo message
	///
	/// These are a few points that can be improved upon in the future.
	///
	/// # Example
	///
	/// ```no_run
	/// let resource = Package {
	///     path: "C:/Program Files (x86)/Steam/steamapps/common/Exanima/Resource.rpk"
	/// };
	/// let opponent03 = resource.load("actors.rpk/opponent03")?;
	/// ```
	pub fn load<'a, P: 'a + AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, EntryError<'a>> {
		let path = path.as_ref();

		// TODO: cleanup redundant code
		match &self.handle {
			Handle::File(file) => {
				let mut reader = Reader::new(file);

				// treat each component of the path as an entry name and traverse the hierarchy
				// of entries until exhausting the iterator of all components then return the
				// bytes associated to the last component.
				let mut components = path.components();
				let mut current_component = components.next();
				let mut previous_offset = 0;
				loop {
					let component = current_component.ok_or(EntryError::InvalidPath {
						expected: "some path".into(),
						found: path.to_owned().into(),
					})?;

					let read_table = read_table(&mut reader)?.with_offset(previous_offset);
					let table_byte_length = read_table.table_byte_length();

					let entry = match component {
						Component::RootDir => continue,
						Component::Normal(os_str) => {
							let mut maybe_entry = Err(EntryError::NotFound {
								name: os_str.display().to_string().into(),
							});
							for entry in read_table {
								let entry = entry?;
								if &*entry.name == os_str {
									maybe_entry = Ok(entry);
									break;
								}
							}
							maybe_entry?
						}
						_ => {
							return Err(EntryError::InvalidPath {
								expected: "format \"path/to/entry\"".into(),
								found: path.to_owned().into(),
							});
						}
					};

					reader.seek(SeekFrom::Start(previous_offset as _))?;
					let asset_offset = entry.asset_offset(table_byte_length);
					reader.seek_relative(asset_offset as _)?;
					previous_offset = previous_offset + asset_offset;

					let next_component = components.next();
					if next_component.is_none() {
						let bytes = entry.read_bytes(&mut reader)?;
						return Ok(bytes);
					}

					current_component = next_component;
				}
			}
			Handle::Directory(package_path) => {
				let mut components = path.components();
				let mut current_component = components.next();
				let mut component_path = package_path.clone();
				loop {
					let component = current_component.ok_or(EntryError::InvalidPath {
						expected: "some path".into(),
						found: path.to_owned().into(),
					})?;

					match component {
						Component::RootDir => continue,
						Component::Normal(_) => {}
						_ => {
							return Err(EntryError::InvalidPath {
								expected: "format \"path/to/entry\"".into(),
								found: path.to_owned().into(),
							});
						}
					}

					component_path.push(component);
					let is_package = if component_path.is_file() {
						let file = File::open(&component_path)?;
						let mut reader = Reader::new(file);
						let mut magic = [0u8; mem::size_of::<u32>()];
						reader.read_bytes(mem::size_of::<u32>(), &mut magic, Default::default())?;
						magic == *Package::MAGIC
					} else {
						false
					};

					if is_package {
						let leftover_path: PathBuf = components.collect();
						let mut leftover_components = leftover_path.components();
						if leftover_components.next().is_some() {
							let package = Package::new(&component_path)?;
							let bytes = package.load(leftover_path).unwrap();

							return Ok(bytes);
						} else {
							// TODO: cleanup redundant code
							let file = File::open(&component_path)?;
							let mut reader = Reader::new(&file);

							let file_name =
								component_path.file_name().ok_or(EntryError::InvalidPath {
									expected: "format \"path/to/entry\"".into(),
									found: component_path.clone().into(),
								})?;
							let file_name = file_name.to_str().ok_or(EntryError::InvalidUtf8(
								file_name.display().to_string().into(),
							))?;

							let entry =
								Entry::new(file_name).with_byte_length(file.metadata()?.len() as _);
							let bytes = entry.read_bytes(&mut reader)?;

							return Ok(bytes);
						};
					}

					let next_component = components.next();
					if next_component.is_none() {
						if component_path.is_dir() {
							todo!("return entries of directory");
						}

						// TODO: cleanup redundant code
						let file = File::open(&component_path)?;
						let mut reader = Reader::new(&file);

						let file_name =
							component_path.file_name().ok_or(EntryError::InvalidPath {
								expected: "format \"path/to/entry\"".into(),
								found: component_path.clone().into(),
							})?;
						let file_name = file_name.to_str().ok_or(EntryError::InvalidUtf8(
							file_name.display().to_string().into(),
						))?;

						let entry =
							Entry::new(file_name).with_byte_length(file.metadata()?.len() as _);
						let bytes = entry.read_bytes(&mut reader)?;

						return Ok(bytes);
					}

					current_component = next_component;
				}
			}
		}
	}
}

pub struct ReadTable<'a, R: Read + Seek> {
	reader: &'a mut Reader<R>,
	table_byte_length: u32,
	offset: u32,
}

impl<R: Read + Seek> ReadTable<'_, R> {
	#[inline]
	pub fn table_byte_length(&self) -> u32 {
		self.table_byte_length
	}

	#[inline]
	pub fn with_offset(mut self, offset: u32) -> Self {
		self.offset = offset;
		self
	}
}

impl<'a, R: Read + Seek> Iterator for ReadTable<'a, R> {
	type Item = Result<Entry, DekuError>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let current_position = match self.reader.stream_position() {
			Ok(v) => v as u32 - self.offset,
			Err(e) => return Some(Err(DekuError::Io(e.kind()))),
		};

		if current_position > self.table_byte_length {
			// no more entries in table, iterator has reached the end so return `None` here
			return None;
		}

		let entry = match Entry::from_reader_with_ctx(&mut self.reader, ()) {
			Ok(v) => v,
			Err(e) => return Some(Err(e)),
		};

		Some(Ok(entry))
	}
}

/// Similar to [`std::fs::read_dir`]
#[inline]
pub fn read_table<'a, R: Read + Seek>(
	reader: &'a mut Reader<R>,
) -> Result<ReadTable<'a, R>, DekuError> {
	let mut magic = [0u8; mem::size_of::<u32>()];
	reader.read_bytes(mem::size_of::<u32>(), &mut magic, Default::default())?;

	if magic != *Package::MAGIC {
		return Err(DekuError::Parse(
			format!(
				"Missing magic value {:#08X}",
				u32::from_be_bytes(*Package::MAGIC)
			)
			.into(),
		)
		.into());
	}

	let mut table_byte_length = [0u8; mem::size_of::<u32>()];
	reader.read_bytes(
		mem::size_of::<u32>(),
		&mut table_byte_length,
		Default::default(),
	)?;
	let table_byte_length = u32::from_le_bytes(table_byte_length);

	Ok(ReadTable {
		reader,
		table_byte_length,
		offset: Default::default(),
	})
}
