use detours_sys::DWORD;
use winapi::{shared::ntdef::NTSTATUS, um::winnt::HANDLE};

#[link(name = "ntdll")]
#[allow(unused)]
extern "system" {
	pub fn NtProtectVirtualMemory(
		ProcessHandle: DWORD,
		BaseAddress: HANDLE,
		NumberOfBytesToProtect: HANDLE,
		NewAccessProtection: DWORD,
		OldAccessProtection: HANDLE,
	) -> NTSTATUS;

	pub fn NtOpenProcess(
		ProcessHandle: HANDLE,
		DesiredAccess: DWORD,
		ObjectAttributes: HANDLE,
		ClientId: HANDLE,
	) -> NTSTATUS;

	pub fn NtQueryVirtualMemory(
		ProcessHandle: DWORD,
		BaseAddress: DWORD,
		MemoryInformationClass: DWORD,
		MemoryInformation: HANDLE,
		MemoryInformationLength: DWORD,
		ReturnLength: HANDLE,
	) -> NTSTATUS;

	pub fn NtCreateSection(
		SectionHandle: *mut HANDLE,
		DesiredAccess: DWORD,
		ObjectAttributes: HANDLE,
		MaximumSize: HANDLE,
		SectionPageProtection: DWORD,
		AllocationAttributes: DWORD,
		FileHandle: HANDLE,
	) -> NTSTATUS;

	pub fn NtUnmapViewOfSection(ProcessHandle: HANDLE, BaseAddress: HANDLE) -> NTSTATUS;

	pub fn NtMapViewOfSection(
		SectionHandle: HANDLE,
		ProcessHandle: HANDLE,
		BaseAddress: HANDLE,
		ZeroBits: DWORD,
		CommitSize: DWORD,
		SectionOffset: HANDLE,
		ViewSize: HANDLE,
		InheritDisposition: DWORD,
		AllocationType: DWORD,
		Win32Protect: DWORD,
	) -> NTSTATUS;

	pub fn ZwProtectVirtualMemory(
		ProcessHandle: HANDLE,
		BaseAddress: HANDLE,
		NumberOfBytesToProtect: HANDLE,
		NewAccessProtection: DWORD,
		OldAccessProtection: HANDLE,
	) -> NTSTATUS;

}

#[repr(i32)]
#[derive(PartialEq, Eq)]
pub enum NtStatus {
	Success = 0x00000000,
	Other(i32),
}

impl From<NTSTATUS> for NtStatus {
	fn from(value: i32) -> Self {
		match value {
			0x00000000 => NtStatus::Success,
			_ => NtStatus::Other(value),
		}
	}
}
