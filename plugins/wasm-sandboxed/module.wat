(module
  (memory (export "memory") 1)
  (global $heap (mut i32) (i32.const 4096))
  (data (i32.const 0) "{\"plugin_id\":\"wasm-sandboxed\",\"action_id\":\"run-demo\",\"title\":\"Hello from Wasmtime\",\"summary\":\"Executed a sandboxed WebAssembly plugin.\",\"success\":true,\"outputs\":[{\"kind\":\"text\",\"title\":\"Sandbox\",\"body\":\"This response came from a WebAssembly module loaded at runtime through Wasmtime.\"}],\"suggested_next_steps\":[\"Compare this with the ABI-stable plugin output.\"]}")
  (func (export "alloc") (param $len i32) (result i32)
    (local $ptr i32)
    global.get $heap
    local.set $ptr
    global.get $heap
    local.get $len
    i32.add
    global.set $heap
    local.get $ptr
  )
  (func (export "invoke_json") (param $ptr i32) (param $len i32) (result i32 i32)
    i32.const 0
    i32.const 362
  )
)
