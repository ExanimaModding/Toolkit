#include "EMF.h"
#include "getFileNameFromHandle.h"

static decltype(&CreateFileW) o_CreateFileW = CreateFileW;

static HANDLE WINAPI CreateFileW_wrap(
	_In_ LPCWSTR lpFileName,
	_In_ DWORD dwDesiredAccess,
	_In_ DWORD dwShareMode,
	_In_opt_ LPSECURITY_ATTRIBUTES lpSecurityAttributes,
	_In_ DWORD dwCreationDisposition,
	_In_ DWORD dwFlagsAndAttributes,
	_In_opt_ HANDLE hTemplateFile)
{
    auto new_path = std::wstring(lpFileName);
    if (new_path.ends_with(L"\\Resource.rpk")) {
        return o_CreateFileW(L"C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima\\mods\\Resource.rpk", dwDesiredAccess, dwShareMode, lpSecurityAttributes, dwCreationDisposition, dwFlagsAndAttributes, hTemplateFile);
    }
	//MessageBoxW(NULL, lpFileName, lpFileName, MB_OK);
	return o_CreateFileW(lpFileName, dwDesiredAccess, dwShareMode, lpSecurityAttributes, dwCreationDisposition, dwFlagsAndAttributes, hTemplateFile);
}

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpReserved) {
  if (DetourIsHelperProcess())
    return 1;

  if (fdwReason == DLL_PROCESS_ATTACH) {
    DetourRestoreAfterWith();

    //parseCommandLineArgs(COMMAND_LINE_ARGS);

    DetourTransactionBegin();
    DetourUpdateThread(GetCurrentThread());

    DetourAttach(&(PVOID&)o_CreateFileW, CreateFileW_wrap);
    //DetourAttach(&(PVOID &)o_GetFileAttributesW, GetFileAttributesW_wrap);
    //DetourAttach(&(PVOID &)o_CreateProcessW, CreateProcessW_wrap);
    MessageBoxW(NULL, L"Hello world", L"Hello world", MB_OK);

    DetourTransactionCommit();
  }

  return 1;
}
