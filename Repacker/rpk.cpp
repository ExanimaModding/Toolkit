#include "rpk.h"

std::vector<unsigned char> RPK::int_to_bytes(int src_int) {
  std::vector<unsigned char> arrayofbyte(4);
  for (int i = 0; i < 4; i++)
    arrayofbyte[i] = (src_int >> (i * 8));
  return arrayofbyte;
}

int RPK::unpack(std::string src, std::string dest) {
  FILE *input_fp;
  errno_t err = fopen_s(&input_fp, src.c_str(), "rb");
  if (err != 0) return err;

  uint32_t sig;
  fread(&sig, sizeof(uint32_t), 1, input_fp);
  if (sig != MAGIC_BYTES) {
    printf("File signature 0x%08x does not match 0x%08x\n", sig, MAGIC_BYTES);
    return 1;
  }

  fs::create_directory(dest);

  uint32_t header_tbl_size_bytes;
  fread(&header_tbl_size_bytes, sizeof(uint32_t), 1, input_fp);
  uint32_t header_tbl_length =
      (uint32_t)std::floor(header_tbl_size_bytes / sizeof(TableEntry));

  std::vector<TableEntry> table(header_tbl_length);
  for (uint32_t i = 0; i < header_tbl_length; i++) {
    TableEntry buf;
    fread(&buf, sizeof(TableEntry), 1, input_fp);
    memcpy(&table[i], &buf, sizeof(TableEntry));
  }

  long long entries_data_start_offset = ftell(input_fp);

  for (TableEntry entry : table) {
    std::string entry_name = to_string(&entry.name);

    _fseeki64(input_fp, entries_data_start_offset + entry.offset, SEEK_SET);

    std::vector<byte> buf(entry.size);
    fread(&buf[0], buf.size(), 1, input_fp);

    FILE *output_fp;

    std::string path = dest;
    path.append("\\");
    path.append(entry_name);

    errno_t err = fopen_s(&output_fp, path.c_str(), "wb");
    if (err != 0) return err;

    fwrite(&buf[0], buf.size(), 1, output_fp);
    fclose(output_fp);
  }

  fclose(input_fp);

  return 0;
}

int RPK::unpack_all(std::string src, std::string dest) {
  for (const auto &entry : fs::directory_iterator(src)) {
    struct _stat64 sb;
    std::string name = entry.path().filename().string();
    std::string path = entry.path().string();

    if (_stat64(path.c_str(), &sb) != 0 || (sb.st_mode & S_IFDIR) ||
        !path.ends_with(".rpk")) {
      continue;
    }

    FILE *input_fp;
    errno_t err = fopen_s(&input_fp, path.c_str(), "rb");
    if (err != 0) return err;

    std::string dest_clone = dest;
    uint32_t ext_pos = name.find_last_of('.');
    std::string dest_child = name.substr(0, ext_pos);
    dest_clone.append("\\");
    dest_clone.append(dest_child);
    unpack(path.c_str(), dest_clone.c_str());

    fclose(input_fp);
  }
  return 0;
}

// Make sure to check within the folder for metadata of the file type
int RPK::pack(std::string src, std::string dest) {
  FILE *output_fp;
  fs::path src_path{src};
  std::string dest_file = src_path.filename().string();
  dest_file.append(".rpk");
  dest.append("\\");
  dest.append(dest_file);

  errno_t err = fopen_s(&output_fp, dest.c_str(), "wb");
  if (err != 0) return err;

  std::vector<byte> buf_magic = int_to_bytes(MAGIC_BYTES);
  fwrite(&buf_magic[0], buf_magic.size(), 1, output_fp);

  uint32_t table_count =
      std::distance(fs::directory_iterator(src), fs::directory_iterator{});
  uint32_t table_size_bytes = table_count * sizeof(TableEntry);

  std::vector<byte> buf_table = int_to_bytes(table_size_bytes);
  fwrite(&buf_table[0], buf_table.size(), 1, output_fp);

  std::vector<std::filesystem::directory_entry> files;
  for (std::filesystem::directory_entry entry : fs::directory_iterator(src)) {
    files.push_back(entry);
  }

  // Sort table entry by name to match the original order.
  // Not matching the original file breaks the game.
  std::sort(files.begin(), files.end(),
            [](std::filesystem::directory_entry a,
               std::filesystem::directory_entry b) {
              return a.path().filename().string() <
                     b.path().filename().string();
            });

  uint32_t offset = 0;
  for (const auto &entry : files) {
    struct stat sb;

    fs::path path = entry.path();
    if (stat(path.string().c_str(), &sb) != 0) continue;
    if ((sb.st_mode & S_IFDIR)) continue;

    TableEntry rpkTableEntry;

    ex_string ex_name = to_ex_string(path.filename().string());

    copy_ex_string(&ex_name, &rpkTableEntry.name);
    rpkTableEntry.offset = offset;
    rpkTableEntry.size = sb.st_size;
    rpkTableEntry._padding1 = 0;
    rpkTableEntry._padding2 = 0;

    fwrite(&rpkTableEntry, sizeof(TableEntry), 1, output_fp);

    offset += sb.st_size;
  }

  for (const auto &entry : files) {
    struct stat sb;

    std::string name = entry.path().filename().string();
    std::string path = entry.path().string();

    if (stat(path.c_str(), &sb) != 0) continue;

    if (sb.st_mode & S_IFDIR) {
      // check for metadata for folder type
      // run class pack function (rdb, rfc, etc.)
      continue;
    }

    std::vector<char> data(sb.st_size);
    std::ifstream input(path.c_str(), std::ios::in | std::ifstream::binary);
    input.read(&data[0], data.size());
    input.close();

    fwrite(&data[0], data.size(), 1, output_fp);
  }

  fclose(output_fp);
  return 0;
}

// Make sure to check within the folder for metadata of the file type
// for now just pack every folder it detects
int RPK::pack_all(std::string src, std::string dest) {
  for (const auto &entry : fs::directory_iterator(src)) {
    struct stat sb;
    std::string name = entry.path().filename().string();
    std::string path = entry.path().string();

    if (stat(path.c_str(), &sb) != 0 || !(sb.st_mode & S_IFDIR)) {
      continue;
    }

    pack(path.c_str(), dest.c_str());
  }
  return 0;
}
