#pragma once

#include <fstream>
#include <nlohmann/json.hpp>
#include <type_traits>

using json = nlohmann::json;

template <class T> // concept
concept is_class = std::is_class<T>::value;

template <is_class T> // use the concept

class Metadata {
public:
  T data;

  static Metadata<T> from(std::string path) {
    std::ifstream f(path);
    Metadata<T> m;
    m.data = json::parse(f);
    return m;
  }

  int save(std::string path) {
    if constexpr (!std::is_convertible_v<
                      T, nlohmann::json_abi_v3_11_2::basic_json>) {
      return 1;
    }
    std::ofstream f(path);
    json j = (json)data;
    f << std::setw(4) << j << std::endl;
    return 0;
  }
};
