# Hot-Reload Example

Watch a plugin library on disk and automatically reload it whenever the
file changes. Useful during development: rebuild in one terminal, see the
new behaviour instantly in another.

## What it demonstrates

- Loading and unloading a shared library at runtime.
- Using `notify` to watch the file system for changes.
- Debouncing rapid-fire events so the file is fully written before reload.
- Caveats of dynamic unloading (OS does not guarantee `dlclose` truly
  unloads the code).

## How to run

**Terminal 1 – start the watcher:**

```bash
cargo build -p hello-world
cargo run -p example-hot-reload -- target/debug/libhello_world.dylib
```

**Terminal 2 – rebuild the plugin:**

```bash
# Edit plugins/hello-world/src/lib.rs, then:
cargo build -p hello-world
```

The watcher in Terminal 1 will detect the rebuilt library and reload it.
