# Gabbro for Zed

Syntax highlighting and language server support for the [Gabbro](https://github.com/gabbro-lang/gabbro)
programming language in the [Zed](https://zed.dev) editor.

- **Highlighting, outline, brackets and indentation** come from the tree-sitter grammar
  [tree-sitter-gabbro](https://github.com/gabbro-lang/tree-sitter-gabbro).
- **Errors, completion, hover and go to definition** come from `gabbro lsp`, the language server that is
  part of the Gabbro compiler. It understands `#import`, so it knows your other files and the standard
  library.

## Install as a dev extension

You need Zed and Rust (installed with rustup, with the `wasm32-wasip1` target).

1. Open Zed, then the command palette, then `zed: install dev extension`.
2. Choose this folder.
3. Open a `.gab` file. Zed builds the extension and the grammar the first time.

## Where the language server comes from

The language server is `gabbro lsp`, a command of the Gabbro compiler. It needs the standard library
that sits next to the compiler, so the whole compiler is used. The extension looks for it in this order:

1. The path in the Zed settings (see below).
2. `gabbro` on the PATH.
3. Otherwise the extension downloads the newest release from
   [gabbro-lang/gabbro](https://github.com/gabbro-lang/gabbro/releases) (about 52 MB, 64 bit Windows) into its
   own folder, shows "Downloading" in Zed, and removes older copies. If GitHub cannot be reached, a copy
   from an earlier run is used.

Use Gabbro 0.2.2 or newer. Older versions analyze each file alone and report errors on every import.

To use a compiler of your own, set its location in the Zed settings. Zed then ignores the arguments of
the extension, so `arguments` must be given too:

```json
{
  "lsp": {
    "gabbro": {
      "binary": {
        "path": "C:/path/to/gabbro.exe",
        "arguments": ["lsp"]
      }
    }
  }
}
```

## Notes

- `extension.toml` names the grammar repository and the exact commit that is used. When the grammar
  changes, update the commit and copy the query files again.
- The query files in `languages/gabbro/` are copies of the ones in the grammar repository. Zed reads
  them from the extension, not from the grammar.

## License

Apache-2.0. See [LICENSE](LICENSE).
