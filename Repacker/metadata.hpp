#pragma once

#include <fstream>
#include <nlohmann/json.hpp>
#include <type_traits>

using json = nlohmann::json;

template <class T>
concept is_class = std::is_class<T>::value;

template <is_class T>

class Metadata {
public:
  T data;

  static Metadata<T> from(std::string path) {
    std::ifstream f(path);
    Metadata<T> m;
    json j = json::parse(f);
    m.data = T{j};
    return m;
  }

  int save(std::string path) {
    std::ofstream f(path);
    json j = (json)data;
    f << std::setw(4) << j << std::endl;
    return 0;
  }
};
