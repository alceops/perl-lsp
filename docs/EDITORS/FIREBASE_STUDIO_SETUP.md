# Firebase Studio Setup Guide for perl-lsp

Firebase Studio runs a VS Code-compatible editor in a remote workspace. That
means `perl-lsp` works well with the same LSP wiring used in VS Code, with a
few remote-environment defaults called out below.

## Prerequisites

- `perllsp` installed in the Studio workspace image and visible on `PATH`
- Perl files opened from the project root folder (not from a parent directory)
- A VS Code-compatible LSP client extension enabled in the workspace

Quick verification inside the Firebase Studio terminal:

```bash
perllsp --version
perllsp --health
which perllsp
```

## Recommended Workspace Settings

Create (or update) `.vscode/settings.json` in your project:

```json
{
  "perl-lsp.serverPath": "perllsp",
  "perl-lsp.autoDownload": false,
  "perl-lsp.trace.server": "off",
  "perl-lsp.enableDiagnostics": true,
  "perl-lsp.enableSemanticTokens": true,
  "perl-lsp.enableFormatting": true,
  "perl-lsp.includePaths": [
    "lib",
    "local/lib/perl5"
  ],
  "files.associations": {
    "*.pl": "perl",
    "*.pm": "perl",
    "*.t": "perl"
  }
}
```

Why this helps in Firebase Studio:

- `serverPath: "perllsp"` avoids absolute host paths that change across
  cloud workspaces.
- `autoDownload: false` prevents startup delays when network policy blocks
  extension downloads.
- Explicit file associations ensure Perl files attach to the right language
  server in mixed-language repos.

## Extension Options

Use either:

1. The official `perl-lsp` extension, or
2. A generic LSP client configured to run `perllsp --stdio`.

If your chosen extension requires explicit command configuration, use:

```json
{
  "command": ["perllsp", "--stdio"]
}
```

## Firebase Studio Troubleshooting

- **"Server not found"**: run `which perllsp`; if empty, install `perllsp` in
  the workspace image and restart the IDE.
- **No diagnostics/completion**: confirm the opened folder is the real project
  root and reload the window.
- **Module resolution misses `lib/` paths**: add project-local include paths in
  `perl-lsp.includePaths`.
- **Slow startup in very large repos**: trim the opened folder scope and review
  [PERFORMANCE_TUNING.md](../how-to/PERFORMANCE_TUNING.md).
