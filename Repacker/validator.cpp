#include "validator.hpp"

std::vector<uint32_t> MAGIC_BYTES_LIST{
    FACTORIES_MAGIC_BYTES_1, FACTORIES_MAGIC_BYTES_2, FTB_MAGIC_BYTES,
    PWR_MAGIC_BYTES,         RFC_MAGIC_BYTES_1,       RFC_MAGIC_BYTES_2,
    RFI_MAGIC_BYTES,         RFP_MAGIC_BYTES,         RFT_MAGIC_BYTES,
    RPK_MAGIC_BYTES,         RSG_MAGIC_BYTES,         RSQ_MAGIC_BYTES,
    WAV_MAGIC_BYTES};

std::string Validator::get_file_extension_from(uint32_t bytes) {
  switch (bytes) {
  // temporarily returning empty string
  // until a name for the file extension
  // presents itself in the future
  case FACTORIES_MAGIC_BYTES_1:
  case FACTORIES_MAGIC_BYTES_2:
    return "";
  case FTB_MAGIC_BYTES:
    return "ftb";
  case PWR_MAGIC_BYTES:
    return "pwr";
  case RFC_MAGIC_BYTES_1:
  case RFC_MAGIC_BYTES_2:
    return "rfc";
  case RFI_MAGIC_BYTES:
    return "rfi";
  case RFP_MAGIC_BYTES:
    return "rfp";
  case RFT_MAGIC_BYTES:
    return "rft";
  case RPK_MAGIC_BYTES:
    return "rpk";
  case RSG_MAGIC_BYTES:
    return "rsg";
  case RSQ_MAGIC_BYTES:
    return "rsq";
  case WAV_MAGIC_BYTES:
    return "wav";
  default:
    return "";
  }
}

uint32_t Validator::get_magic_bytes_from(std::string path) {
  FILE *fp;
  errno_t err = fopen_s(&fp, path.c_str(), "rb");
  if (err != 0) throw err;

  uint32_t sig;
  fread(&sig, sizeof(uint32_t), 1, fp);
  fclose(fp);
  return sig;
}

bool Validator::is_magic_bytes_valid(uint32_t bytes) {
  bool is_valid = false;
  for (uint32_t MAGIC_BYTES : MAGIC_BYTES_LIST) {
    if (bytes == MAGIC_BYTES) {
      is_valid = true;
      break;
    }
  }
  return is_valid;
}
