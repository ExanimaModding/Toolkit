#include "string_utils.hpp"

void copy_ex_string(ex_string *src, ex_string *dest) {
  for (int i = 0; i < 16; i++) {
    dest->string[i] = src->string[i];
  }
}

std::string validate_string(std::string src, bool strip_ext) {
  std::string dest;
  if (strip_ext || src.length() > 16) {
    uint32_t ext_pos = src.find_last_of('.');
    if (ext_pos > 16 || ext_pos == std::string::npos) {
      printf("Make filename '%s' 16 characters or less.\n"
             "File extensions (e.g. '.rfi') do not count "
             "towards character count.\n",
             src.c_str());
      throw std::invalid_argument("Invalid filename");
    }
    if (!strip_ext) {
      // packing Objlib doesn't strip extensions
      printf("string_utils.cpp: Possible bug with filename, '%s'\n",
             src.c_str());
    }
    dest = src.substr(0, ext_pos);
  } else {
    dest = src;
  }
  return dest;
}

std::string to_string(ex_string *src) {
  std::string dest;
  for (int i = 0; i < 16; i++) {
    if (src->string[i] != 0) {
      dest.push_back(src->string[i]);
    }
  }
  return dest;
}

ex_string to_ex_string(std::string src, bool strip_ext) {
  std::string validated_str = validate_string(src, strip_ext);
  ex_string dest;
  for (int i = 0; i < 16; i++) {
    if (i > validated_str.length() - 1) {
      dest.string[i] = 0;
    } else {
      dest.string[i] = validated_str[i];
    }
  }
  return dest;
}
