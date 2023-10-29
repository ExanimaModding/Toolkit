#include <iostream>
#include <stdio.h>
#include <string.h>
#include <vector>
#include <windows.h>
#include <filesystem>
#include <sys/stat.h>
#include <fstream>
#include "string_utils.h"

namespace fs = std::filesystem;

#define MAGIC_BYTES 0xAFBF0C01

struct RPKTableEntry {
  ex_string name;
  uint32_t offset;
  uint32_t size;
  uint32_t _padding1;
  uint32_t _padding2;
};

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

int unpack(std::string src, std::string dest) {
  FILE *input_fp;
  errno_t result = fopen_s(&input_fp, src.c_str(), "rb");
  if (result != 0) {
    return result;
  }

  uint32_t sig;
  fread(&sig, sizeof(uint32_t), 1, input_fp);
  if (sig != MAGIC_BYTES) {
    printf("File signature 0x%08x does not match 0x%08x\n", sig, MAGIC_BYTES);
    return 1;
  }

  uint32_t table_size_bytes;
  fread(&table_size_bytes, sizeof(uint32_t), 1, input_fp);

  fs::create_directory(dest);

  std::vector<RPKTableEntry> table((uint32_t)std::floor(table_size_bytes / 32));
  table.resize((uint32_t)std::floor(table_size_bytes / 32));
  for (uint32_t i = 0; i < (uint32_t)std::floor(table_size_bytes / 32); i++) {
    uint32_t buf[8];
    fread(&buf, sizeof(uint32_t[8]), 1, input_fp);
    memcpy(&table[i], &buf, sizeof(RPKTableEntry));
  }

  uint32_t data_start = ftell(input_fp);
  for (RPKTableEntry &entry : table) {
    std::string name = to_string(&entry.name);

    uint32_t *data = (uint32_t *)malloc(entry.size);
    if (data == NULL) {
      printf("Failed to malloc\n");
      return 1;
    }

    fseek(input_fp, data_start + entry.offset, SEEK_SET);
    fread(data, entry.size, 1, input_fp);

    FILE *output_fp;
    std::string path = dest;
    path.append("\\");
    path.append(name);
    printf("%s\n", path.c_str());
    errno_t err = fopen_s(&output_fp, path.c_str(), "wb");
    if (err != 0) return err;

    fwrite(data, entry.size, 1, output_fp);
    fclose(output_fp);
    free(data);
  }

  //fclose(input_fp);
  return 0;
}


int pack(std::string src, std::string dest) {
  FILE *output_fp;
  fs::path src_path{ src };
  std::string dest_file = src_path.filename().string();
  dest_file.append(".rpk");
  dest.append("\\");
  dest.append(dest_file);

  errno_t result = fopen_s(&output_fp, dest.c_str(), "wb");
  if (result != 0) return result;

  std::vector<unsigned char> buf_magic = int_to_bytes(MAGIC_BYTES);
  fwrite(&buf_magic[0], buf_magic.size(), 1, output_fp);

  uint32_t table_count = std::distance(fs::directory_iterator(src), fs::directory_iterator{});
  uint32_t table_size_bytes = table_count * sizeof(RPKTableEntry);

  std::vector<unsigned char> buf_table = int_to_bytes(table_size_bytes);
  fwrite(&buf_table[0], buf_table.size(), 1, output_fp);

  uint32_t table_start = ftell(output_fp);
  uint32_t origin_data = table_start + table_size_bytes;

  std::vector<char> padding(table_size_bytes);
  fwrite(&padding[0], table_size_bytes, 1, output_fp);
  fflush(output_fp);

  std::vector<std::filesystem::directory_entry> files;
  struct stat sb;
  for (std::filesystem::directory_entry entry : fs::directory_iterator(src)) {
    files.push_back(entry);
  }

  std::sort(
    files.begin(),
    files.end(),
    [](std::filesystem::directory_entry a, std::filesystem::directory_entry b) {
      return a.path().filename().string() < b.path().filename().string();
    }
  );

  std::vector<RPKTableEntry> table(table_count);
  uint32_t i = 0;
  for (const auto& entry : files) {
    RPKTableEntry rpk_entry;
    std::string name = entry.path().filename().string();
    std::string path = entry.path().string();
    if (stat(path.c_str(), &sb) != 0) {
      i++;
      continue;
    }

    if (!(sb.st_mode & S_IFDIR)) {
      ex_string ex_name = to_ex_string(name);
      copy_ex_string(&ex_name, &rpk_entry.name);
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

      fseek(output_fp, rpk_entry.offset + origin_data, SEEK_SET);
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

//int unpack_all() {
//  FILE* input_fp;
//  char src[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
//               "\\Exanima\\";
//  for (const auto& entry : fs::directory_iterator(src)) {
//    std::string name = entry.path().filename().string();
//    std::string path = entry.path().string();
//	char dest[] = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
//				  "\\Exanima\\mods\\";
//    unpack(input_fp, path);
//  }
//}

//int pack_all() {}

// unpack() and pack() both output corrupted data
int main() {
  //std::string src_unpack = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
		//				   "\\Exanima\\Textures.rpk";
  //std::string dest_unpack = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
		//				    "\\Exanima\\unpacked\\Textures";
  //int result = unpack(src_unpack, dest_unpack);
  //if (result != 0) {
  //  return result;
  //}


  std::string src_pack = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
						 "\\Exanima\\unpacked\\Textures";
  std::string dest_pack = "C:\\Program Files (x86)\\Steam\\steamapps\\common"
						  "\\Exanima\\packed";
  return pack(src_pack, dest_pack);

}
