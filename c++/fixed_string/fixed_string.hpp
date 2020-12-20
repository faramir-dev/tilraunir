#pragma once

#include "common.hpp"

template <$::size_t N>
struct FixedString {
    char content_[N];
    $::size_t real_length_ = 0;

    constexpr FixedString(const char (&str)[N]) {
        for ($::size_t i = 0; i < N && 0 != str[i]; ++i, ++real_length_) {
            content_[i] = str[i];
        }
    }

    constexpr $::size_t size() const noexcept { return real_length_; }
};

template<char... CHS> struct VarString{};

namespace detail {

template <$::size_t IDX, FixedString STR, char... CHS>
static inline constexpr auto to_var_string_impl() {
    if constexpr (0 == IDX) {
        return VarString<CHS...>{};
    } else {
        return to_var_string_impl<IDX - 1, STR, STR.content_[IDX - 1], CHS...>();
    }
}

}

template <FixedString STR>
static inline constexpr auto to_var_string() {
    return detail::to_var_string_impl<STR.size(), STR>();
}