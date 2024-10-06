use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

macro_rules! define_languages {
    ($(($variant:ident, $name:expr, [$($alias:expr),*])),* $(,)?) => {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
        pub enum Languages {
            $(
                #[serde(rename = $name)]
                $variant,
            )*
        }

        impl std::fmt::Display for Languages {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out = match self {
                    $(Languages::$variant => $name,)*
                };
                write!(f, "{out}")
            }
        }

        impl Languages {
            pub fn from_codeblock_language(codeblock: &str) -> Option<Self> {
                match codeblock {
                    $($(
                        $alias => Some(Self::$variant),
                    )*)*
                    _ => None,
                }
            }

            pub fn enabled(&self) -> bool {
                CONFIG.languages.contains(self)
            }

            pub fn disabled_languages() -> Vec<Languages> {
                LANGUAGES.iter()
                    .filter(|&lang| !CONFIG.languages.contains(lang))
                    .cloned()
                    .collect()
            }
        }

        pub const LANGUAGES: &[Languages] = &[
            $(Languages::$variant,)*
        ];
    }
}

pub const FOOBAR: &[Languages] = &[Languages::Bash, Languages::C, Languages::Clojure];

// Usage
define_languages!(
    (Apl, "apl", ["apl"]),
    (Bash, "bash", ["bash", "sh", "zsh"]),
    (BrainFuck, "brainfuck", ["brainfuck", "bf"]),
    (C, "c", ["c", "h"]),
    (Clojure, "clojure", ["clojure", "clj"]),
    (
        Cpp,
        "cpp",
        ["cpp", "hpp", "cc", "hh", "c++", "h++", "cxx", "hxx"]
    ),
    (Crystal, "crystal", ["crystal", "cr"]),
    (CSharp, "csharp", ["csharp", "cs"]),
    (Elixir, "elixir", ["elixir", "ex"]),
    (FSharp, "fsharp", ["fsharp", "fs", "fsx", "fsi", "fsscript"]),
    (Golang, "golang", ["golang", "go"]),
    (Haskell, "haskell", ["haskell", "hs"]),
    (Idris, "idris", ["idris"]),
    (Java, "java", ["java"]),
    (JavaScript, "javascript", ["javascript", "js"]),
    (Julia, "julia", ["julia", "jl"]),
    (Lua, "lua", ["lua"]),
    (OCaml, "ocaml", ["ocaml", "ml"]),
    (Pascal, "pascal", ["pascal"]),
    (Perl, "perl", ["perl", "pl", "pm"]),
    (Php, "php", ["php"]),
    (Prolog, "prolog", ["prolog"]),
    (Python, "python", ["python", "py"]),
    (Racket, "racket", ["racket"]),
    (Ruby, "ruby", ["ruby", "rb"]),
    (Rust, "rust", ["rust", "rs"]),
    (SQL, "sql", ["sql"]),
    (TypeScript, "typescript", ["typescript", "ts"]),
);
