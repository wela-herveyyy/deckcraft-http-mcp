# deckcraft-http-mcp
HTTP remote MCP server for DeckCraft with multi-model AI support (Claude, ChatGPT, Gemini, Cursor, Antigravity, etc)

## Usage

```bash
cargo run -- --port 3000 --sample
cargo run -- --mode connect --connect 7777      # attach to `deckcraft --control 7777`
```

Endpoints: `POST /mcp/tools/list`, `POST /mcp/tools/call`, `GET /mcp/describe/{tool}`,
`GET /mcp/resources`, `POST /antigravity/execute`, `POST /mcp` (JSON-RPC), `GET /health`.
Request formats (MCP, OpenAI, Codex, Gemini, Cursor/LSP, Antigravity, JSON-RPC) are auto-detected and
responses are returned in the same format. `tools/list` accepts `?format=openai|codex|gemini|antigravity`.

Connected mode sends newline-delimited JSON-RPC `tools/call` over TCP to the control port.
`--automation-read-root` / `--automation-write-root` confine `open_deck` / `save_deck` in headless mode.
