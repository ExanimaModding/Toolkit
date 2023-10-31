#pragma once

#include "../metadata.hpp"
#include "../string_utils.hpp"
#include "../validator.hpp"
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

  struct Meta {
    std::string filetype = "rpk";
    bool use_file_extensions = false;

    Meta() {}

    explicit Meta(json data) {
      filetype = data["filetype"];
      use_file_extensions = data["use_file_extensions"];
    }

    operator json() {
      return {{"filetype", filetype},
              {"use_file_extensions", use_file_extensions}};
    }
  };

  static std::vector<unsigned char> int_to_bytes(int src_int);

  static int unpack(std::string src, std::string dest);

  static int unpack_all(std::string src, std::string dest);

  static int pack(std::string src, std::string dest);

  static int pack_all(std::string src, std::string dest);
};
