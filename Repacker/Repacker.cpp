#include <iostream>
#include <stdio.h>
#include <string.h>
#include <vector>
#include <windows.h>
#include <filesystem>
#include <sys/stat.h>
#include <fstream>

namespace fs = std::filesystem;

#define MAGIC_BYTES 0xAFBF0C01

struct RPKTableEntry {
  char name[16];
  uint32_t offset;
  uint32_t size;
  uint32_t _padding1;
  uint32_t _padding2;
};

//struct RPKFile {
//  uint32_t sig;
//  uint32_t table_size;
//  std::vector<RPKTableEntry> table;
//};

void copy_string(const char *src, char *dest, uint32_t length) {
    for (int i = 0; i < length; i++) {
      dest[i] = src[i];
    }
}

int unpack(FILE* fp, char* dest) {
  uint32_t sig;
  fread(&sig, sizeof(uint32_t), 1, fp);
  if (sig != MAGIC_BYTES) {
    printf("File signature 0x%08x does not match 0x%08x\n", sig, MAGIC_BYTES);
    return 1;
  }

  uint32_t table_size;
  fread(&table_size, sizeof(uint32_t), 1, fp);

  std::vector<RPKTableEntry> table((uint32_t)std::floor(table_size / 32));
  for (uint32_t i = 0; i < (uint32_t)std::floor(table_size / 32); i++) {
    uint32_t buf[8];
    fread(&buf, sizeof(uint32_t[8]), 1, fp);
    memcpy(&table[i], &buf, sizeof(RPKTableEntry));
  }

  int data_start = ftell(fp);
  for (const RPKTableEntry &entry : table) {
    char terminated_name[17] = {0};
    copy_string(entry.name, terminated_name, 16);
    //for (int i = 0; i < 16; i++) {
    //  terminated_name[i] = entry.name[i];
    //}
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
    sprintf_s(new_name, "%s\\%s", dest, terminated_name);

    errno_t err = fopen_s(&unpacked, new_name, "wb");
    if (err != 0) return err;

    fwrite(data, entry.size, 1, unpacked);
    free(data);
  }

  return 0;
}

int pack() {
  char src[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
               "\\Exanima\\mods\\Apparel";
  char dest[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
                "\\Exanima\\mods\\repacked";

  FILE *fp;
  std::string dest_file{ dest };
  dest_file.append("\\Apparel.rpk");
  //fopen_s(&fp, dest_file.c_str(), "wb");
  std::ofstream fout;
  fout.open(dest_file.c_str(), std::ofstream::out | std::ofstream::binary | std::ofstream::app);
  uint32_t sig = MAGIC_BYTES; // written the wrong way around, may be an issue
  fout.write(reinterpret_cast<char*>(&sig), sizeof(sig));
  uint32_t table_items = std::distance(fs::directory_iterator(src), fs::directory_iterator{});
  printf("%d\n", table_items);
  uint32_t table_size = table_items * sizeof(RPKTableEntry);
  printf("%d %d\n", table_items, sizeof(RPKTableEntry));
  fout.write(reinterpret_cast<char*>(&table_size), sizeof(table_size)); // writing the wrong number
  uint32_t table_start = fout.tellp();
  fout.seekp(table_items * sizeof(RPKTableEntry));

  std::vector<RPKTableEntry> table(table_items);
  struct stat sb;
  for (const auto& entry : fs::directory_iterator(src)) {
    RPKTableEntry rpk_entry;
    //fs::path path = entry.path();
    std::string name = entry.path().filename().string();
    if (name.length() > 16) {
      uint32_t ext_pos = name.find_last_of('.');
      if (ext_pos > 16 || ext_pos == std::string::npos) {
        printf("Make filename '%s' 16 characters or less.\n"
               "File extensions (e.g. '.rfi') do not count towards character count.", name.c_str());
        return 1;
      }
      //name = name.substr(0, ext_pos);
      std::string stripped_name = name.substr(0, ext_pos);
      copy_string(stripped_name.c_str(), rpk_entry.name, stripped_name.length());
      printf("%s\n", rpk_entry.name);
	 // for (int i = 0; i < stripped_name.length(); i++) {
		//rpk_entry.name[i] = stripped_name[i];
	 // }

      //printf("%s\n", stripped_name.c_str());
    }
    else {
      copy_string(name.c_str(), rpk_entry.name, name.length());
      printf("%s\n", rpk_entry.name);
	 // for (int i = 0; i < name.length(); i++) {
		//rpk_entry.name[i] = name[i];
	 // }
    }

    std::string path = entry.path().string();
    if (stat(path.c_str(), &sb) != 0) continue;

    if (!(sb.st_mode & S_IFDIR)) {
      // read first 4 bytes and check if valid exanima file
      //printf("%s\n", path.c_str());

	  // open file, read data, write to struct
	  FILE *packed;
	  std::string src_file{ src };
	  src_file.append(rpk_entry.name);
	  errno_t err = fopen_s(&packed, src_file.c_str(), "rb");
	  if (err != 0) return err;

      rpk_entry.offset = fout.tellp();
      rpk_entry.size = sb.st_size;
      rpk_entry._padding1 = 0;
      rpk_entry._padding2 = 0;

      table.push_back(rpk_entry);

	  uint32_t *buf = new uint32_t [sb.st_size];
	  fread(buf, sb.st_size, 1, packed);
      fout << buf; // not being written at all maybe because of seeking
      delete[] buf;

    }
    else {
      // check for metadata for folder type
      // run class pack function (rdb, rfc, etc.)
    }
  }

  fout.close();
  return 0;
}

int main() {
  //FILE *fp;
  //char src[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
  //             "\\Exanima\\mods\\Resource.rpk";
  //char dest[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
  //              "\\Exanima\\mods\\Resource";
  //errno_t err = fopen_s(&fp, src, "rb");
  //if (err != 0) {
  //  return err;
  //}

  //unpack(fp, dest);
  //fclose(fp);

  //return 1;

  return pack();
}
