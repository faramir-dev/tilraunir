/*
 * $ g++-10 -Ofast -std=c++20 -c main.cxx
 */


#include "fixed_string.hpp"

int
main() {
    constexpr auto s0 = FixedString("Dia dhuit");
    constexpr auto s1 = to_var_string<s0>();
    constexpr auto s2 = to_var_string<"Ahoj svete">();
    constexpr auto s3 = to_var_string<"שלום עולם">();
}
