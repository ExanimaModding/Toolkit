#pragma once

#include <filesystem>
#include <vector>

const uint32_t FACTORIES_MAGIC_BYTES_1 = 0xAFCE0F00;
const uint32_t FACTORIES_MAGIC_BYTES_2 = 0xAFCE0F01;
const uint32_t FTB_MAGIC_BYTES = 0x3EEFBD01;
const uint32_t PWR_MAGIC_BYTES = 0xAFCE01CE;
const uint32_t RFC_MAGIC_BYTES_1 = 0x3D23AFCF;
const uint32_t RFC_MAGIC_BYTES_2 = 0x3D21AFCF;
const uint32_t RFI_MAGIC_BYTES = 0x1D2D3DC6;
const uint32_t RFP_MAGIC_BYTES = 0xAFDFBD10;
const uint32_t RFT_MAGIC_BYTES = 0x3EEFAD01;
const uint32_t RPK_MAGIC_BYTES = 0xAFBF0C01;
const uint32_t RSG_MAGIC_BYTES = 0xDA7AEA02;
const uint32_t RSQ_MAGIC_BYTES = 0x3D000000;
const uint32_t WAV_MAGIC_BYTES = 0x46464952;

class Validator {
public:
  static std::string get_file_extension_from(uint32_t bytes);

  static uint32_t get_magic_bytes_from(std::string path);

  static bool is_magic_bytes_valid(uint32_t bytes);
};
