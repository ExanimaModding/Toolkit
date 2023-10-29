#pragma once

#include <string>
#include <stdexcept>

typedef struct ex_string { char string[16]; } ex_string;

void copy_ex_string(ex_string* src, ex_string* dest);

std::string validate_string(std::string src, bool strip_ext = false);

std::string to_string(ex_string *src);

ex_string to_ex_string(std::string src, bool strip_ext = false);
