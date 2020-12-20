##### FIxed `constexpr` strings and `template<char...>`

- `FixedString` -- It is `constexpr` string-like type that stores characters in a buffer.
- `VarString<char...>` -- It stores characters as template arguments.
- `to_var_string` -- Function (`constexpr`) that converts `FixedString` to `VarString`.