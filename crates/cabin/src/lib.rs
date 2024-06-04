/// The `compile_time` module. Although lang2 is a compiled language, it has the ability to run
/// arbitrary code at compile time, which requires an `compile_time`.
pub mod compile_time;

/// The lexer module, which tokenizes source code into a stream of tokens.
pub mod lexer;

/// The parser module, which parses a stream of tokens into an abstract syntax tree.
pub mod parser;

/// The scopes module, which manages the scope of variables and functions.
pub mod scopes;

/// The context module, which manages global state of the compiler.
pub mod context;

/// The "C Runner" module. This module handles transpiling ASTs to C code, compiling C code, running C code, removing compiled C code, etc.
/// Basically everything after the `compile_time` step is going to go in here.
pub mod compiler;

/// The formatter module. This handles code formatting for Cabin code. The Cabin formatter is un-opinionated, and provides no configuration options. The formatting
/// process is fairly straightforward; The code is parsed and then the AST is recursively turned back into Cabin code. Essentially, it's a transpiler into itself.
pub mod formatter;

/// The CLI module. This module handles tooling related to the CLI, such as pretty-printing code snippets and errors, configuration options, subcommands, etc.
pub mod cli;

/// The `util` module. This module handles utility operations like number formatting.
pub mod util;

/// The Cabin prelude. This is a string of cabin code that's appended automatically to the beginning of all Cabin files prior to compilation. It includes basic necessities such as
/// IO, file handling, basic data types like strings and numbers, etc.
pub const PRELUDE: &str = include_str!("../prelude.cbn");
