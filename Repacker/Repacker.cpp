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
      //if (src[i] == 0xCC) {
      //  dest[i] = 0;
      //}
      //else {
	  dest[i] = src[i];
      //}
    }
}

std::vector<unsigned char> int_to_bytes(int src_int)
{
  std::vector<unsigned char> arrayofbyte(4);
  for (int i = 0; i < 4; i++)
    arrayofbyte[i] = (src_int >> (i * 8));
  return arrayofbyte;
}

void readFromFile(std::vector<int>& numbers, std::ifstream& infile) {
  int num;
  while (infile >> num)
    numbers.push_back(num);
}

std::vector<unsigned char> get_contents(const std::string& path) {
  if (std::ifstream source_file{ path, std::ios::binary }; source_file) {
    return std::vector<byte>(std::istreambuf_iterator<char>{source_file}, {});
  }

  std::cerr << "Unable to correctly open file \" << path <<\".\n";

  return {};
}

int unpack(FILE* fp, char* dest) {
  uint32_t sig;
  fread(&sig, sizeof(uint32_t), 1, fp);
  if (sig != MAGIC_BYTES) {
    printf("File signature 0x%08x does not match 0x%08x\n", sig, MAGIC_BYTES);
    return 1;
  }

  uint32_t table_size_bytes;
  fread(&table_size_bytes, sizeof(uint32_t), 1, fp);

  std::vector<RPKTableEntry> table((uint32_t)std::floor(table_size_bytes / 32));
  table.resize((uint32_t)std::floor(table_size_bytes / 32));
  for (uint32_t i = 0; i < (uint32_t)std::floor(table_size_bytes / 32); i++) {
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

// print out each letter every time we use copy_string to find the problem
// string utils in different file
// string for name is getting messed up
int pack() {
  char src[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
               "\\Exanima\\mods\\Apparel\\";
  char dest[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
                "\\Exanima\\mods\\repacked\\";

  FILE *output_fp;
  std::string dest_file{ dest };
  dest_file.append("Apparel.rpk");

  errno_t err = fopen_s(&output_fp, dest_file.c_str(), "wb");
  if (err != 0) return err;

  std::vector<unsigned char> buf_magic = int_to_bytes(MAGIC_BYTES);
  fwrite(&buf_magic[0], buf_magic.size(), 1, output_fp);

  uint32_t table_count = std::distance(fs::directory_iterator(src), fs::directory_iterator{});
  uint32_t table_size_bytes = table_count * sizeof(RPKTableEntry);

  std::vector<unsigned char> buf_table = int_to_bytes(table_size_bytes);
  fwrite(&buf_table[0], buf_table.size(), 1, output_fp);

  uint32_t table_start = ftell(output_fp);
  uint32_t origin_data = table_size_bytes + 8;

  std::vector<char> padding(table_size_bytes);
  fwrite(&padding[0], table_size_bytes, 1, output_fp);
  fflush(output_fp);

  std::vector<RPKTableEntry> table(table_count);
  struct stat sb;
  uint32_t i = 0;
  for (const auto& entry : fs::directory_iterator(src)) {
    RPKTableEntry rpk_entry;
    std::string name = entry.path().filename().string();
    if (name.length() > 16) {
      uint32_t ext_pos = name.find_last_of('.');
      if (ext_pos > 16 || ext_pos == std::string::npos) {
        printf("Make filename '%s' 16 characters or less.\n"
               "File extensions (e.g. '.rfi') do not count towards character count.", name.c_str());
        return 1;
      }
      std::string stripped_name = name.substr(0, ext_pos);
      copy_string(stripped_name.c_str(), rpk_entry.name, stripped_name.length());
    }
    else {
      copy_string(name.c_str(), rpk_entry.name, name.length());
    }
	printf("%s\n", rpk_entry.name);

    std::string path = entry.path().string();
    if (stat(path.c_str(), &sb) != 0) {
      i++;
      continue;
    }

    if (!(sb.st_mode & S_IFDIR)) {
	  FILE *input_fp;
	  std::string src_file{ src };

	  char terminated_name[17] = {0};
	  copy_string(rpk_entry.name, terminated_name, 16);
      for (int i = 0; i < 16; i++) {
        printf("%d\n", terminated_name[i]);
        if (terminated_name[i] == 0 || terminated_name[i] == -52) {
          terminated_name[i] = '\0';
        }
      }
	  terminated_name[16] = '\0';
      copy_string(terminated_name, rpk_entry.name, 16);

	  src_file.append(name);

      rpk_entry.offset = ftell(output_fp) - origin_data;

      rpk_entry.size = sb.st_size;
      rpk_entry._padding1 = 0;
      rpk_entry._padding2 = 0;

      table.push_back(rpk_entry);

      std::vector<char> data;
      data.resize(sb.st_size);
      std::ifstream input(path.c_str(), std::ios::in | std::ifstream::binary);

      input.read(&data[0], data.size());
      input.close();

      fwrite(&data[0], data.size(), 1, output_fp);
      fflush(output_fp);
      memcpy(&table[i], &rpk_entry, sizeof(RPKTableEntry));
    }
    else {
      // check for metadata for folder type
      // run class pack function (rdb, rfc, etc.)
    }
    i++;
  }
  fseek(output_fp, table_start, SEEK_SET);
  fwrite(&table[0], sizeof(RPKTableEntry)* table_count, 1, output_fp);

  fclose(output_fp);
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
