#include "emf.h"
#include "get_file_from_handle.h"
#include <conio.h>
#include <future>
#include <iostream>
#include <thread>

static decltype(&CreateFileW) o_CreateFileW = CreateFileW;
typedef void (*EntryPointFn)();
EntryPointFn o_start;

static HANDLE WINAPI CreateFileW_wrap(
    _In_ LPCWSTR lpFileName, _In_ DWORD dwDesiredAccess, _In_ DWORD dwShareMode,
    _In_opt_ LPSECURITY_ATTRIBUTES lpSecurityAttributes,
    _In_ DWORD dwCreationDisposition, _In_ DWORD dwFlagsAndAttributes,
    _In_opt_ HANDLE hTemplateFile) {
  auto new_path = std::wstring(lpFileName);
  if (new_path.ends_with(L"\\Resource.rpk")) {
    return o_CreateFileW(
        L"C:\\Program Files "
        L"(x86)\\Steam\\steamapps\\common\\Exanima\\mods\\Resource.rpk",
        dwDesiredAccess, dwShareMode, lpSecurityAttributes,
        dwCreationDisposition, dwFlagsAndAttributes, hTemplateFile);
  }
  // MessageBoxW(NULL, lpFileName, lpFileName, MB_OK);
  return o_CreateFileW(lpFileName, dwDesiredAccess, dwShareMode,
                       lpSecurityAttributes, dwCreationDisposition,
                       dwFlagsAndAttributes, hTemplateFile);
}

static void start_wrap() {
  LPVOID moduleHandle = GetModuleHandleA(NULL);

  LPVOID aEventPtr = ((char *)moduleHandle - 0x400000 + 0x004CDFC0);
  LPVOID aActor = ((char *)moduleHandle + 0x005FA3F4 - 0x400000);

  AllocConsole();
  FILE *fDummy;
  freopen_s(&fDummy, "CONIN$", "r", stdin);
  freopen_s(&fDummy, "CONOUT$", "w", stderr);
  freopen_s(&fDummy, "CONOUT$", "w", stdout);

  printf("moduleHandle: %#08x\naEventPtr: %#08x\n", moduleHandle, aEventPtr);

  printf("aEvent: %s\n", (char *)aEventPtr); // crashes here

  // LPVOID unknownFunction = SigScanner::search("\x53\x83\x78\x34\x00",
  // "xxxxx", (PBYTE)((char*)moduleHandle - 0x400000), 0x605000-0x4cd000);

  // char SIG[] = "53 83 78 34 00";
  // char MASK[] = "xxxxx";

  // UINT_PTR unknownFunction = (UINT_PTR)Pattern::In::Scan(SIG, MASK,
  // (char*)moduleHandle - 0x400000, sizeof(MASK) - 1);

  LPVOID getHealth = ((char *)moduleHandle + 0x005F9F34 - 0x400000);
  LPVOID healthPtr = (LPVOID)0x00642AC4;
  LPVOID currentLevelPtr = ((char *)moduleHandle - 0x400000 + 0x637180);

  printf("fnptr: %#08x\n", getHealth);
  printf("modHandle: %#08x\n", moduleHandle);

  MessageBoxA(NULL, (char *)aEventPtr, "balls", MB_OK);

  typedef uint32_t(__fastcall * fGetHealth)();

  auto fnGetHealth = (fGetHealth)getHealth;

  char x = 0;

  auto proc = GetCurrentProcess();

  using namespace std::literals;
  auto f = std::async(std::launch::async, [&] {
    while (1) {
      // auto result = ReadProcessMemory(proc, currentLevelPtr, &x, sizeof(x),
      // 0);
      char mapLevel = *(char *)currentLevelPtr;
      auto err = GetLastError();
      printf("Map Level: %d, Error code: %d, Last Error: %d\n", x, mapLevel,
             err);

      try {

        // *(**off_642ac4 + 0x0B48) / *(**off_642ac4 + 0x0DE0)
        auto ref_healthPtr = *reinterpret_cast<void ***>(0x642ac4);
        if (ref_healthPtr == NULL) continue;
        auto healthPtr = *ref_healthPtr;
        if (healthPtr == NULL) continue;

        auto ref_redHealthPtr = *reinterpret_cast<void ***>(0x642ac4);
        if (ref_redHealthPtr == NULL) continue;
        auto redHealthPtr = *ref_redHealthPtr;
        if (redHealthPtr == NULL) continue;

        auto currentHealth = reinterpret_cast<float *>((char *)healthPtr + 0x0B30);
        auto maxHealth = reinterpret_cast<float *>((char *)healthPtr + 0x0B44);

        auto red0 = reinterpret_cast<float *>((char *)healthPtr + 0xB34);

        auto currentHealth_readOnly =
            reinterpret_cast<float *>((char *)healthPtr + 0x0b48);

        auto currentMana =
            reinterpret_cast<float *>((char *)redHealthPtr + 0x0b4c);

        // currentHealth_readOnly = (currentHealth - maxHealth) * unknown + maxHealth;

        // c = (2 - 2) * 1 + 2
        // (a - b) * c + b

        auto v15 = *currentHealth_readOnly / *maxHealth;
        printf("Health: %f=%f/%f, = (%f - %f) * unkn + %f\n", v15,
               *currentHealth_readOnly, *maxHealth, *currentHealth, *maxHealth, *currentHealth);
        printf("Red Health: %f / %f\n", *red0, *maxHealth);

        float newHealth = 0.25;
        float newMana = 1.0;
        // memcpy_s(currentHealth_readOnly, sizeof(float), &newHealth, sizeof(float));
        memcpy_s(currentHealth, sizeof(float), &newHealth, sizeof(float));
        memcpy_s(maxHealth, sizeof(float), &newHealth, sizeof(float));
        memcpy_s(currentMana, sizeof(float), &newMana, sizeof(float));
        memcpy_s(red0, sizeof(float), &newHealth, sizeof(float));

      } catch (int err) {
        printf("Error: %d", err);
      }
      std::this_thread::sleep_for(1s);
    }
  });

  o_start();
}

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpReserved) {
  if (DetourIsHelperProcess()) return 1;

  if (fdwReason == DLL_PROCESS_ATTACH) {
    DetourRestoreAfterWith();

    DetourTransactionBegin();
    DetourUpdateThread(GetCurrentThread());

    PVOID entryPointPtr = DetourGetEntryPoint(NULL);
    o_start = (EntryPointFn)entryPointPtr;
    DetourAttach(&(PVOID &)o_start, start_wrap);

    // MessageBoxA(NULL, "Hello world", (char*)p, MB_OK);

    DetourTransactionCommit();
  }

  return 1;
}
