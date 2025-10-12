(program 1.0.0
 [(builtin equalsInteger)
  [(builtin byteStringToInteger) (con bool False) (con bytestring #123456abcdef)]
  [(builtin byteStringToInteger) (con bool False) (con bytestring #123456abcdef0000000000000000)]
 ]
)
