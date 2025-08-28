# MOO Database Parser References

## Official C Implementations

### ToastStunt
- Repository: https://github.com/lisdude/toaststunt
- Database parser: https://github.com/lisdude/toaststunt/blob/master/src/db_file.cc
- Database I/O: https://github.com/lisdude/toaststunt/blob/master/src/db_io.cc
- Key insights:
  - Uses `dbio_scanf()` with whitespace skipping between fields
  - Validates each number ends with newline: `if (isspace(*s) || *p != '\n')`
  - Special handling for verb program terminator: `'.'` followed by newline

### LambdaMOO
- Repository: https://github.com/wrog/lambdamoo
- Database parser: https://github.com/wrog/lambdamoo/blob/main/db_file.c
- Database I/O: https://github.com/wrog/lambdamoo/blob/main/db_io.c
- Key insights:
  - `dbio_scanf()` explicitly skips whitespace with `while (isspace(c))`
  - String reading strips trailing newlines
  - Both implementations are flexible with whitespace between sections

## Parser Implementation Notes

The C implementations show that the parser should be flexible with whitespace between sections. Instead of expecting exactly one newline, we should use `many0(line_ending)` or similar to consume any number of newlines between major sections.

## Testing Databases

- Minimal.db - 4 objects, basic test case
- LambdaCore-latest.db - 97 objects, standard core
- JHCore-DEV-2.db - 237 objects, has extra newline after header
- toastcore.db - Also fails to parse, likely similar formatting variations