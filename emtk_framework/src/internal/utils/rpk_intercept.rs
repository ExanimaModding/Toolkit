use std::{
	collections::{BTreeMap, HashSet},
	ffi::{CString, c_void},
	fs,
	io::{Cursor, Read},
	mem,
	path::PathBuf,
	ptr,
	sync::{LazyLock, Mutex},
};

use detours_sys::{DetourAttach, DetourTransactionBegin, DetourTransactionCommit};
use emtk_asset::{
	Entry, Package,
	deku::{DekuWriter, writer::Writer},
};
use tracing::error;
use winapi::{
	shared::minwindef::{BOOL, DWORD, LPDWORD, LPVOID, MAX_PATH},
	um::{
		fileapi::{GetFinalPathNameByHandleA, ReadFile, SetFilePointer},
		minwinbase::LPOVERLAPPED,
		winbase::{FILE_BEGIN, FILE_CURRENT},
		winnt::HANDLE,
	},
};

use crate::MOD_ENTRIES;

#[derive(Debug)]
struct FileState {
	/// End of the table, from the start of the file.
	pub original_end_of_table: u32,
	/// End of the table, from the start of the file.
	pub intercepted_end_of_table: u32,
	pub entries: Vec<SomeEntry>,
}

#[derive(Debug)]
struct SomeEntry {
	pub original_offset: u32,
	#[allow(unused)]
	pub original_size: u32,
	pub intercepted_offset: u32,
	pub intercepted_size: u32,
	pub is_foreign: bool,
	pub is_modified: bool,
	pub asset_name: String,
}

static PACKAGES: LazyLock<Mutex<BTreeMap<PathBuf, FileState>>> =
	LazyLock::new(|| Mutex::new(BTreeMap::new()));

static mut O_READ_FILE: *mut c_void = 0 as _;

pub unsafe fn register_hooks() {
	unsafe {
		O_READ_FILE = ReadFile as *mut c_void;

		DetourTransactionBegin();
		DetourAttach(&raw mut O_READ_FILE, read_file as _);
		DetourTransactionCommit();
	}
}

type TReadFile = unsafe extern "system" fn(
	h_file: HANDLE,
	lp_buffer: LPVOID,
	n_number_of_bytes_to_read: DWORD,
	lp_number_of_bytes_read: LPDWORD,
	lp_overlapped: LPOVERLAPPED,
) -> BOOL;

unsafe fn read_file(
	h_file: HANDLE,
	lp_buffer: LPVOID,
	n_number_of_bytes_to_read: DWORD,
	lp_number_of_bytes_read: LPDWORD,
	lp_overlapped: LPOVERLAPPED,
) -> BOOL {
	static READ_FILE: LazyLock<TReadFile> =
		LazyLock::new(|| unsafe { mem::transmute(O_READ_FILE) });

	'try_proxy_file: {
		// Get the file name from the handle.
		let file_name: PathBuf = {
			let mut file_name = vec![0u8; MAX_PATH];
			let len = unsafe {
				GetFinalPathNameByHandleA(
					h_file,
					file_name.as_mut_ptr() as *mut i8,
					MAX_PATH as u32,
					0,
				)
			};
			file_name.truncate(len as usize);
			let Ok(file_name) = CString::new(file_name) else {
				break 'try_proxy_file;
			};
			let Ok(file_name) = file_name.to_str() else {
				break 'try_proxy_file;
			};
			PathBuf::from(file_name.to_string())
		};

		if file_name.extension().is_none_or(|ext| ext != "rpk") {
			break 'try_proxy_file;
		}

		let requested_offset = unsafe { SetFilePointer(h_file, 0, ptr::null_mut(), FILE_CURRENT) };

		// Lock the global packages map for the duration of this function.
		let mut packages = PACKAGES.lock().unwrap();

		// [Magic::RPK, table_size]
		if requested_offset == 0 && n_number_of_bytes_to_read == 8 {
			let Ok(original_rpk) = Package::new(&file_name).map_err(|e| error!("{e}")) else {
				break 'try_proxy_file;
			};
			let Ok(mut original_entries) = original_rpk.entries().map_err(|e| error!("{e}")) else {
				break 'try_proxy_file;
			};

			// Get all of the new entries (not in the original RPK)
			// and modified entries (in the original RPK, but modified).
			let (new_entries, modified_entries) = {
				let existing_names: HashSet<_> =
					original_entries.iter().map(|e| e.name.clone()).collect();

				let Some(file_name_without_ext) = file_name.file_stem() else {
					break 'try_proxy_file;
				};
				let mod_entries = MOD_ENTRIES
					.get()
					.unwrap()
					.get(&file_name_without_ext.display().to_string())
					.unwrap();
				let new_entries: Vec<_> = mod_entries
					.iter()
					.filter_map(|(_, path)| {
						let Some(Some(file_name)) = path.file_name().map(|p| p.to_str()) else {
							return None;
						};
						if !existing_names.contains(file_name) {
							Some(path)
						} else {
							None
						}
					})
					.collect();

				let modified_entries: Vec<_> = mod_entries
					.iter()
					.filter_map(|(_, path)| {
						let Some(Some(file_name)) = path.file_name().map(|p| p.to_str()) else {
							return None;
						};
						if existing_names.contains(file_name) {
							Some(path)
						} else {
							None
						}
					})
					.collect();

				(new_entries, modified_entries)
			};

			let original_table_size_bytes = original_entries.len() * Entry::RAW_SIZE;
			let mut state = FileState {
				// magic (4) + table size (4) + entry_bytes (entries_count*32)
				original_end_of_table: 8 + original_table_size_bytes as u32,
				// defined later
				intercepted_end_of_table: 0,
				entries: vec![],
			};

			// We are completely recalculating the offsets and sizes of all entries.
			let mut previous_offset = 0;
			let mut previous_size = 0;

			// IMPORTANT: Sort the original entries by offset to ensure they are in the correct order.
			// Exanima will break if we don't do this.
			original_entries.sort_by(|a, b| a.byte_offset.cmp(&b.byte_offset));

			// Loop through every entry in the original RPK and check if it has been modified.
			for entry in original_entries.iter() {
				if let Some(updated) = modified_entries.iter().find(|path| {
					path.file_name().and_then(|name| name.to_str()) == Some(&entry.name)
				}) {
					// If the file has been modified, record the original offset and size.
					// Then, read the new file and get the size, and set the new offset.
					let size = updated.metadata().unwrap().len() as u32;
					let offset = previous_offset + previous_size;

					state.entries.push(SomeEntry {
						original_offset: entry.byte_offset,
						original_size: entry.byte_length,
						intercepted_offset: offset,
						intercepted_size: size,
						is_foreign: false,
						is_modified: true,
						asset_name: entry.name.clone().into(),
					});

					previous_offset = offset;
					previous_size = size;
				} else {
					// If the file is unmodified, store the original offset and size.
					// Then, set the new offset. The size is unchanged.
					let offset = previous_offset + previous_size;
					let size = entry.byte_length;

					state.entries.push(SomeEntry {
						original_offset: entry.byte_offset,
						original_size: entry.byte_length,
						intercepted_offset: offset,
						intercepted_size: size,
						is_foreign: false,
						is_modified: false,
						asset_name: entry.name.clone().into(),
					});

					previous_offset = offset;
					previous_size = size;
				}
			}

			// Now, we loop through all the new entries and add them to the end of the table.
			for entry in new_entries.iter() {
				let size = entry.metadata().unwrap().len() as u32;
				let offset = previous_offset + previous_size;

				state.entries.push(SomeEntry {
					original_offset: u32::MAX,
					original_size: u32::MAX,
					intercepted_offset: offset,
					intercepted_size: size,
					is_foreign: true,
					is_modified: false,
					asset_name: entry.file_name().unwrap().to_string_lossy().into_owned(),
				});

				previous_offset = offset;
				previous_size = size;
			}

			// IMPORTANT: Finally, we sort the entries by name.
			// Exanima will break if we don't do this.
			state
				.entries
				.sort_by(|a, b| a.asset_name.cmp(&b.asset_name));

			// Each entry is 32 bytes.
			let table_size_bytes = state.entries.len() as u32 * 32;

			// (Entries*32) + magic (4) + table size (4)
			state.intercepted_end_of_table = table_size_bytes as u32 + 8;

			// insert into the global packages map.
			packages.insert(file_name, state);

			// Write out our ReadFile results for the caller to read.
			let buffer = lp_buffer as *mut u32;
			unsafe {
				*buffer = u32::from_le_bytes(*Package::MAGIC);
				*(buffer.offset(1)) = table_size_bytes;
				*lp_number_of_bytes_read = 8;

				// Increment the file pointer, as is the default behaviour of ReadFile.
				SetFilePointer(h_file, 8, ptr::null_mut(), FILE_BEGIN);
			}

			return 1;
		} else if requested_offset == 8 && packages.contains_key(&file_name) {
			// [table header]
			let state = packages.get(&file_name).expect("Failed to get file state");

			let mut buffer: Vec<[u8; Entry::RAW_SIZE]> =
				Vec::with_capacity(state.entries.len() * Entry::RAW_SIZE);
			for entry in &state.entries {
				let new_entry = Entry {
					name: entry.asset_name.clone().into(),
					byte_offset: entry.intercepted_offset,
					byte_length: entry.intercepted_size,
				};
				let mut bytes = Cursor::new([0u8; Entry::RAW_SIZE]);
				if new_entry
					.to_writer(&mut Writer::new(&mut bytes), ())
					.map_err(|e| error!("{e}"))
					.is_err()
				{
					break 'try_proxy_file;
				}
				buffer.push(bytes.into_inner());
			}
			let buffer = buffer.into_iter().flatten().collect::<Vec<_>>();

			unsafe {
				ptr::copy_nonoverlapping(buffer.as_ptr(), lp_buffer as _, buffer.len() as _);
				*lp_number_of_bytes_read = buffer.len() as u32;

				SetFilePointer(h_file, buffer.len() as i32, ptr::null_mut(), FILE_CURRENT);
			}

			return 1;
		} else if packages.contains_key(&file_name) {
			// [Asset Data]
			let state = packages.get(&file_name).expect("Failed to get file state");

			// Deduct the table header size from the requested offset.
			let requested_offset = requested_offset - state.intercepted_end_of_table;

			// Find the entry that matches the requested offset.
			let intercepted_entry = state.entries.iter().find(|entry| {
				let range =
					entry.intercepted_offset..entry.intercepted_offset + entry.intercepted_size;
				range.contains(&(requested_offset as u32))
			});

			if let Some(entry) = intercepted_entry {
				if entry.is_foreign || entry.is_modified {
					let Some(file_name_without_ext) = file_name.file_stem() else {
						break 'try_proxy_file;
					};
					let mod_entries = MOD_ENTRIES
						.get()
						.unwrap()
						.get(&file_name_without_ext.display().to_string())
						.unwrap();
					let file_path = mod_entries.get(&entry.asset_name).unwrap();

					let mut file = fs::File::open(file_path).unwrap();
					let mut buffer = vec![0u8; entry.intercepted_size as usize];
					file.read_exact(&mut buffer).unwrap();

					unsafe {
						ptr::copy_nonoverlapping(
							buffer.as_ptr(),
							lp_buffer as _,
							buffer.len() as _,
						);
						*lp_number_of_bytes_read = buffer.len() as u32;
					}
					return 1;
				}

				unsafe {
					// Set the file pointer to the start of the original entry.
					SetFilePointer(
						h_file,
						entry.original_offset as i32 + state.original_end_of_table as i32,
						ptr::null_mut(),
						FILE_BEGIN,
					);

					// Read the original entry.
					let result = READ_FILE(
						h_file,
						lp_buffer,
						n_number_of_bytes_to_read,
						lp_number_of_bytes_read,
						lp_overlapped,
					);

					// Set the file pointer to where exanima expects it to be.
					// This is intercepted offset + intercepted size.
					SetFilePointer(
						h_file,
						(requested_offset + n_number_of_bytes_to_read) as _,
						ptr::null_mut(),
						FILE_BEGIN,
					);

					// Return the result of the ReadFile call.
					return result;
				}
			} else {
				panic!("Failed to find matching intercepted entry for offset: {requested_offset}");
			}
		}
	}

	unsafe {
		READ_FILE(
			h_file,
			lp_buffer,
			n_number_of_bytes_to_read,
			lp_number_of_bytes_read,
			lp_overlapped,
		)
	}
}
