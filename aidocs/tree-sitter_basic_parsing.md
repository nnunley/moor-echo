Basic Parsing - Tree-sitter



* Light
* Rust
* Coal
* Navy
* Ayu

Tree-sitter
===========



[Basic Parsing](#basic-parsing "#basic-parsing")
================================================

[Providing the Code](#providing-the-code "#providing-the-code")
---------------------------------------------------------------

In the example on the previous page, we parsed source code stored in a simple string using the `ts_parser_parse_string` function:

```
TSTree *ts_parser_parse_string(
  TSParser *self,
  const TSTree *old_tree,
  const char *string,
  uint32_t length
);

```

You may want to parse source code that's stored in a custom data structure, like a [piece table](https://en.wikipedia.org/wiki/Piece_table "https://en.wikipedia.org/wiki/Piece_table") or a [rope](https://en.wikipedia.org/wiki/Rope_(data_structure) "https://en.wikipedia.org/wiki/Rope_(data_structure)").
In this case, you can use the more general `ts_parser_parse` function:

```
TSTree *ts_parser_parse(
  TSParser *self,
  const TSTree *old_tree,
  TSInput input
);

```

The `TSInput` structure lets you provide your own function for reading a chunk of text at a given byte offset and row/column
position. The function can return text encoded in either UTF-8 or UTF-16. This interface allows you to efficiently parse
text that is stored in your own data structure.

```
typedef struct {
  void *payload;
  const char *(*read)(
    void *payload,
    uint32_t byte_offset,
    TSPoint position,
    uint32_t *bytes_read
  );
  TSInputEncoding encoding;
  DecodeFunction decode;
} TSInput;

```

If you want to decode text that is not encoded in UTF-8 or UTF-16, you can set the `decode` field of the input to your function
that will decode text. The signature of the `DecodeFunction` is as follows:

```
typedef uint32_t (*DecodeFunction)(
  const uint8_t *string,
  uint32_t length,
  int32_t *code_point
);

```

Attention

The `TSInputEncoding` must be set to `TSInputEncodingCustom` for the `decode` function to be called.

The `string` argument is a pointer to the text to decode, which comes from the `read` function, and the `length` argument
is the length of the `string`. The `code_point` argument is a pointer to an integer that represents the decoded code point,
and should be written to in your `decode` callback. The function should return the number of bytes decoded.

[Syntax Nodes](#syntax-nodes "#syntax-nodes")
---------------------------------------------

Tree-sitter provides a [DOM](https://en.wikipedia.org/wiki/Document_Object_Model "https://en.wikipedia.org/wiki/Document_Object_Model")-style interface for inspecting syntax trees.
A syntax node's *type* is a string that indicates which grammar rule the node represents.

```
const char *ts_node_type(TSNode);

```

Syntax nodes store their position in the source code both in raw bytes and row/column
coordinates. In a point, rows and columns are zero-based. The `row` field represents
the number of newlines before a given position, while `column` represents the number
of bytes between the position and beginning of the line.

```
uint32_t ts_node_start_byte(TSNode);
uint32_t ts_node_end_byte(TSNode);
typedef struct {
  uint32_t row;
  uint32_t column;
} TSPoint;
TSPoint ts_node_start_point(TSNode);
TSPoint ts_node_end_point(TSNode);

```

[Retrieving Nodes](#retrieving-nodes "#retrieving-nodes")
---------------------------------------------------------

Every tree has a *root node*:

```
TSNode ts_tree_root_node(const TSTree *);

```

Once you have a node, you can access the node's children:

```
uint32_t ts_node_child_count(TSNode);
TSNode ts_node_child(TSNode, uint32_t);

```

You can also access its siblings and parent:

```
TSNode ts_node_next_sibling(TSNode);
TSNode ts_node_prev_sibling(TSNode);
TSNode ts_node_parent(TSNode);

```

These methods may all return a *null node* to indicate, for example, that a node does not *have* a next sibling.
You can check if a node is null:

```
bool ts_node_is_null(TSNode);

```

[Named vs Anonymous Nodes](#named-vs-anonymous-nodes "#named-vs-anonymous-nodes")
---------------------------------------------------------------------------------

Tree-sitter produces [*concrete* syntax trees](https://en.wikipedia.org/wiki/Parse_tree "https://en.wikipedia.org/wiki/Parse_tree") — trees that contain nodes for
every individual token in the source code, including things like commas and parentheses. This is important for use-cases
that deal with individual tokens, like [syntax highlighting](https://en.wikipedia.org/wiki/Syntax_highlighting "https://en.wikipedia.org/wiki/Syntax_highlighting"). But some
types of code analysis are easier to perform using an [*abstract* syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree "https://en.wikipedia.org/wiki/Abstract_syntax_tree") — a tree in which the less important
details have been removed. Tree-sitter's trees support these use cases by making a distinction between
*named* and *anonymous* nodes.

Consider a grammar rule like this:

```
if_statement: $ => seq("if", "(", $._expression, ")", $._statement);

```

A syntax node representing an `if_statement` in this language would have 5 children: the condition expression, the body statement,
as well as the `if`, `(`, and `)` tokens. The expression and the statement would be marked as *named* nodes, because they
have been given explicit names in the grammar. But the `if`, `(`, and `)` nodes would *not* be named nodes, because they
are represented in the grammar as simple strings.

You can check whether any given node is named:

```
bool ts_node_is_named(TSNode);

```

When traversing the tree, you can also choose to skip over anonymous nodes by using the `_named_` variants of all of the
methods described above:

```
TSNode ts_node_named_child(TSNode, uint32_t);
uint32_t ts_node_named_child_count(TSNode);
TSNode ts_node_next_named_sibling(TSNode);
TSNode ts_node_prev_named_sibling(TSNode);

```

If you use this group of methods, the syntax tree functions much like an abstract syntax tree.

[Node Field Names](#node-field-names "#node-field-names")
---------------------------------------------------------

To make syntax nodes easier to analyze, many grammars assign unique *field names* to particular child nodes.
In the [creating parsers](../creating-parsers/3-writing-the-grammar.html#using-fields "../creating-parsers/3-writing-the-grammar.html#using-fields") section, it's explained how to do this in your own grammars. If a syntax node has
fields, you can access its children using their field name:

```
TSNode ts_node_child_by_field_name(
  TSNode self,
  const char *field_name,
  uint32_t field_name_length
);

```

Fields also have numeric ids that you can use, if you want to avoid repeated string comparisons. You can convert between
strings and ids using the `TSLanguage`:

```
uint32_t ts_language_field_count(const TSLanguage *);
const char *ts_language_field_name_for_id(const TSLanguage *, TSFieldId);
TSFieldId ts_language_field_id_for_name(const TSLanguage *, const char *, uint32_t);

```

The field ids can be used in place of the name:

```
TSNode ts_node_child_by_field_id(TSNode, TSFieldId);

```
