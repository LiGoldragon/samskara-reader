# samskara-reader

Read-only MCP server for samskara's world.db. Provides query access to all
world relations without write or commit capability. Used by agents in other
repos (via `nix develop .#reader`) for DB queries.

## VCS

Jujutsu (`jj`) is mandatory. Git is the backend only. Always pass `-m` to
`jj` commands.
