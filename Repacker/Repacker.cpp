#include <iostream>
#include <stdio.h>
#include <string.h>
#include <vector>
#include <windows.h>

#define MAGIC_BYTES 0xAFBF0C01

struct RPKTableEntry {
  char name[16];
  uint32_t offset;
  uint32_t size;
  uint32_t _padding1;
  uint32_t _padding2;
};

int unpack(std::string src, std::string dest) {
  FILE *fp;
  errno_t err = fopen_s(&fp, src.c_str(), "rb");
  if (err != 0) {
    return err;
  }

  uint32_t sig;
  fread(&sig, sizeof(uint32_t), 1, fp);
  if (sig != MAGIC_BYTES) {
    printf("File signature 0x%08x does not match 0x%08x\n", sig, MAGIC_BYTES);
    return 1;
  }

  uint32_t table_size;
  fread(&table_size, sizeof(uint32_t), 1, fp);

  std::vector<RPKTableEntry> table(std::floor(table_size / 32));
  for (uint32_t i = 0; i < (uint32_t)std::floor(table_size / 32); i++) {
    uint32_t buf[8];
    fread(&buf, sizeof(uint32_t[8]), 1, fp);
    memcpy(&table[i], &buf, sizeof(RPKTableEntry));
  }

  int data_start = ftell(fp);
  for (const RPKTableEntry &entry : table) {
    char terminated_name[17] = {0};
    for (int i = 0; i < 16; i++) {
      terminated_name[i] = entry.name[i];
    }
    terminated_name[16] = '\0';

    uint32_t *data = (uint32_t *)malloc(entry.size);
    if (data == NULL) {
      printf("Failed to malloc\n");
      return 1;
    }

    fseek(fp, data_start + entry.offset, SEEK_SET);
    fread(data, entry.size, 1, fp);

    FILE *unpacked;
    char new_name[MAX_PATH];
    sprintf_s(new_name, "%s\\%s", dest.c_str(), terminated_name);

    errno_t err = fopen_s(&unpacked, new_name, "wb");
    if (err != 0) {
      return err;
    }

    fwrite(data, entry.size, 1, unpacked);
    fclose(unpacked);
    free(data);
  }

  fclose(fp);
  return 0;
}

int pack(std::string src, std::string dest) { return 0; }

int main() {
  return unpack("C:\\Program Files (x86)\\Steam\\steamapps\\common\\"
                "Exanima\\mods\\Resource.rpk",
                "C:\\Program Files (x86)\\Steam\\steamapps\\common\\"
                "Exanima\\mods\\Resource");
}
