use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

pub const LANGUAGES: &[&str] = &[
    "apl",
    "bash",
    "brainfuck",
    "c",
    "clojure",
    "cpp",
    "csharp",
    "elixir",
    "fsharp",
    "golang",
    "haskell",
    "idris",
    "java",
    "javascript",
    "julia",
    "lua",
    "ocaml",
    "pascal",
    "perl",
    "php",
    "prolog",
    "python",
    "racket",
    "ruby",
    "rust",
    "typescript",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Languages {
    #[serde(rename = "apl")]
    Apl,
    #[serde(rename = "bash")]
    Bash,
    #[serde(rename = "brainfuck")]
    BrainFuck,
    #[serde(rename = "c")]
    C,
    #[serde(rename = "clojure")]
    Clojure,
    #[serde(rename = "cpp")]
    Cpp,
    #[serde(rename = "csharp")]
    CSharp,
    #[serde(rename = "elixir")]
    Elixir,
    #[serde(rename = "fsharp")]
    FSharp,
    #[serde(rename = "golang")]
    Golang,
    #[serde(rename = "haskell")]
    Haskell,
    #[serde(rename = "idris")]
    Idris,
    #[serde(rename = "java")]
    Java,
    #[serde(rename = "javascript")]
    JavaScript,
    #[serde(rename = "julia")]
    Julia,
    #[serde(rename = "lua")]
    Lua,
    #[serde(rename = "ocaml")]
    OCaml,
    #[serde(rename = "pascal")]
    Pascal,
    #[serde(rename = "perl")]
    Perl,
    #[serde(rename = "php")]
    Php,
    #[serde(rename = "prolog")]
    Prolog,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "racket")]
    Racket,
    #[serde(rename = "ruby")]
    Ruby,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "typescript")]
    TypeScript,
}

impl std::fmt::Display for Languages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Languages::Apl => "apl".to_string(),
            Languages::Bash => "bash".to_string(),
            Languages::BrainFuck => "brainfuck".to_string(),
            Languages::C => "c".to_string(),
            Languages::Clojure => "clojure".to_string(),
            Languages::Cpp => "cpp".to_string(),
            Languages::CSharp => "csharp".to_string(),
            Languages::Elixir => "elixir".to_string(),
            Languages::FSharp => "fsharp".to_string(),
            Languages::Golang => "go".to_string(),
            Languages::Haskell => "haskell".to_string(),
            Languages::Idris => "idris".to_string(),
            Languages::Java => "java".to_string(),
            Languages::JavaScript => "javascript".to_string(),
            Languages::Julia => "julia".to_string(),
            Languages::Lua => "lua".to_string(),
            Languages::OCaml => "ocaml".to_string(),
            Languages::Pascal => "pascal".to_string(),
            Languages::Perl => "perl".to_string(),
            Languages::Php => "php".to_string(),
            Languages::Prolog => "prolog".to_string(),
            Languages::Python => "python".to_string(),
            Languages::Racket => "racket".to_string(),
            Languages::Ruby => "ruby".to_string(),
            Languages::Rust => "rust".to_string(),
            Languages::TypeScript => "typescript".to_string(),
        };
        write!(f, "{out}")
    }
}

impl Languages {
    pub fn from_codeblock_language(codeblock: &str) -> Option<Self> {
        match codeblock {
            "apl" => Some(Self::Apl),
            "bash" | "sh" | "zsh" => Some(Self::Bash),
            "brainfuck" | "bf" => Some(Self::BrainFuck),
            "c" | "h" => Some(Self::C),
            "clojure" | "clj" => Some(Self::Clojure),
            "cpp" | "hpp" | "cc" | "hh" | "c++" | "h++" | "cxx" | "hxx" => Some(Self::Cpp),
            "csharp" | "cs" => Some(Self::CSharp),
            "elixir" | "ex" => Some(Self::Elixir),
            "fsharp" | "fs" | "fsx" | "fsi" | "fsscript" => Some(Self::FSharp),
            "golang" | "go" => Some(Self::Golang),
            "haskell" | "hs" => Some(Self::Haskell),
            "idris" => Some(Self::Idris),
            "java" => Some(Self::Java),
            "javascript" | "js" => Some(Self::JavaScript),
            "julia" | "jl" => Some(Self::Julia),
            "lua" => Some(Self::Lua),
            "ocaml" | "ml" => Some(Self::OCaml),
            "pascal" => Some(Self::Pascal),
            "perl" | "pl" | "pm" => Some(Self::Perl),
            "php" => Some(Self::Php),
            "prolog" => Some(Self::Prolog),
            "python" | "py" => Some(Self::Python),
            "racket" => Some(Self::Racket),
            "ruby" | "rb" => Some(Self::Ruby),
            "rust" | "rs" => Some(Self::Rust),
            "typescript" | "ts" => Some(Self::TypeScript),
            _ => None,
        }
    }

    pub fn enabled(&self) -> bool {
        CONFIG.languages.contains(&self.to_string())
    }
}
