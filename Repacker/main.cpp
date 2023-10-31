#include "formats/rpk.hpp"
#include <string.h>

int main() {
  // string_utils.cpp in validate_string(): packing Objlib doesn't strip
  // extensions
  // rpk.cpp in unpack(): stool_brass c2. in Objlib.rpk is causing
  // this to flip
  // rpk.cpp in pack(): path.ends_with("rdb") and
  // path.ends_with("rcd") not working

  // clang-format off
  //std::string src_unpack = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\"
  //                         "Exanima\\Objlib.rpk";
  //std::string dest_unpack =
  //    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\"
  //    "Exanima\\unpacked\\Objlib";
  //int result_unpack = RPK::unpack(src_unpack, dest_unpack);
  //if (result_unpack != 0) {
  //  return result_unpack;
  //}

  // clang-format off
  std::string src_unpack_all =
      "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima";
  std::string dest_unpack_all =
      "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima\\unpacked";
  int result_unpack_all = RPK::unpack_all(src_unpack_all, dest_unpack_all);
  if (result_unpack_all != 0) {
    return result_unpack_all;
  }

  // clang-format off
  //std::string src_pack = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\"
  //                       "Exanima\\unpacked\\Resource";
  //std::string dest_pack =
  //    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima\\packed";
  //return RPK::pack(src_pack, dest_pack);

  // clang-format off
  std::string src_pack_all =
      "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima\\unpacked";
  std::string dest_pack_all =
      "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima\\packed";
  return RPK::pack_all(src_pack_all, dest_pack_all);
}
