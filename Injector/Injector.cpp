#include "injector.h"

int hookExecutable(std::string executable) {
  auto pInjectDllPath = std::string("EMF.dll");
  auto szInjectDllFullPath = std::filesystem::absolute(pInjectDllPath).string();

  PROCESS_INFORMATION ProcessInfo;
  memset((void *)&ProcessInfo, 0, sizeof(ProcessInfo));

  STARTUPINFOA StartupInfo;
  memset((void *)&StartupInfo, 0, sizeof(StartupInfo));
  StartupInfo.cb = sizeof(StartupInfo);

  if (DetourCreateProcessWithDllExA(
          NULL, executable.data(), NULL, NULL, 0, CREATE_SUSPENDED, NULL, NULL,
          &StartupInfo, &ProcessInfo, szInjectDllFullPath.data(), NULL) == 0) {
    printf("Failed to create process\n");
    return 1;
  }

  ResumeThread(ProcessInfo.hThread);

  CloseHandle(ProcessInfo.hProcess);
  CloseHandle(ProcessInfo.hThread);

  return 0;
}

int main() {
  return hookExecutable("c:\\Program Files (x86)\\Steam\\steamapps\\common\\"
                        "Exanima\\Exanima.exe");
}
