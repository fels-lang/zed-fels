# Fels for Zed

Fels was called Gabbro until version 0.3.1; this extension was zed-gabbro.

Syntax highlighting and language server support for the [Fels](https://github.com/fels-lang/fels)
programming language in the [Zed](https://zed.dev) editor.

- **Highlighting, outline, brackets and indentation** come from the tree-sitter grammar
  [tree-sitter-fels](https://github.com/fels-lang/tree-sitter-fels).
- **Errors, completion, hover and go to definition** come from `fels lsp`, the language server that is
  part of the Fels compiler. It understands `#import`, so it knows your other files and the standard
  library.

## Install as a dev extension

You need Zed and Rust (installed with rustup, with the `wasm32-wasip1` target).

1. Open Zed, then the command palette, then `zed: install dev extension`.
2. Choose this folder.
3. Open a `.fels` file. Zed builds the extension and the grammar the first time.

## Where the language server comes from

The language server is `fels lsp`, a command of the Fels compiler. It needs the standard library
that sits next to the compiler, so the whole compiler is used. The extension looks for it in this order:

1. The path in the Zed settings (see below).
2. `fels` on the PATH.
3. Otherwise the extension downloads the newest release from
   [fels-lang/fels](https://github.com/fels-lang/fels/releases) (about 52 MB, 64 bit Windows) into its
   own folder, shows "Downloading" in Zed, and removes older copies. If GitHub cannot be reached, a copy
   from an earlier run is used.

Use Fels, or Gabbro 0.2.2 or newer. Older versions analyze each file alone and report errors on every import.

To use a compiler of your own, set its location in the Zed settings. Zed then ignores the arguments of
the extension, so `arguments` must be given too:

```json
{
  "lsp": {
    "fels": {
      "binary": {
        "path": "C:/path/to/fels.exe",
        "arguments": ["lsp"]
      }
    }
  }
}
```

## Notes

- `extension.toml` names the grammar repository and the exact commit that is used. When the grammar
  changes, update the commit and copy the query files again.
- The query files in `languages/fels/` are copies of the ones in the grammar repository. Zed reads
  them from the extension, not from the grammar.

## License

Apache-2.0. See [LICENSE](LICENSE).
