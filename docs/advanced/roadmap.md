# Advanced Track Roadmap

The workspace already includes crate slots for the next two major architecture tracks:

- `crates/plugin-abi`
- `crates/plugin-wasm`

## ABI-stable native plugins

This track is for:

- compatibility-oriented native extension contracts
- longer-lived plugin ecosystems
- version negotiation and interface evolution

## WASM sandboxed plugins

This track is for:

- stronger runtime isolation
- portability
- safer execution of less-trusted extensions
- web/service-oriented plugin workflows

## Current status

The repository currently has the full workspace structure and the working native JSON path in place. ABI-stable and WASM tracks are scaffolded next so the repo can grow without a structural rewrite.

