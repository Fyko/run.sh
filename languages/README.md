# Adding a new language

## 1. Docker Image

Each language is isolated in its own Docker image. Each image has a script `run.sh` that accepts the code as an argument.
Look at some of the existing images to see how they work.

## 2. Adding the language to the codebase

Luckily, you don't have to do many changes to the codebase to add a new language.

Preserving alphabetical order, add the language to:

1. the `LANGUAGES` constant in [`src/hypervisor/languages.rs`](../src/hypervisor/languages.rs).
2. the `Languages` enum in [`src/hypervisor/languages.rs`](../src/hypervisor/languages.rs).
3. the `std::fmt::Display` implementation for `Languages` in [`src/hypervisor/languages.rs`](../src/hypervisor/languages.rs).
4. the `from_codeblock_language` function in [`src/hypervisor/languages.rs`](../src/hypervisor/languages.rs). Use the [Highlight.js documentation](https://highlightjs.readthedocs.io/en/latest/supported-languages.html#supported-languages)
   to find the aliases for the language.

## 4. Create a PR

Great! Now that you're done, create a PR and wait for it to be merged.
