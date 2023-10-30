#pragma once

#include "string_utils.h"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <vector>
#include <windows.h>

namespace fs = std::filesystem;

class RPK {
public:
#pragma pack(1)
  struct TableEntry {
    ex_string name;
    uint32_t offset;
    uint32_t size;
    uint32_t _padding1;
    uint32_t _padding2;
  };
#pragma pack(0)

  static const uint32_t MAGIC_BYTES = 0xAFBF0C01;

  static std::vector<unsigned char> int_to_bytes(int src_int);

  static int unpack(std::string src, std::string dest);

  static int unpack_all(std::string src, std::string dest);

  static int pack(std::string src, std::string dest);

  static int pack_all(std::string src, std::string dest);
};
