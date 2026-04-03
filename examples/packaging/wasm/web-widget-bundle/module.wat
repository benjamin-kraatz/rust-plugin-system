(module
  (memory (export "memory") 1)
  (global $heap (mut i32) (i32.const 4096))
  (data (i32.const 0) "{\"plugin_id\":\"web-widget\",\"action_id\":\"render-widget\",\"title\":\"WASM Widget Output\",\"summary\":\"Returned widget-oriented output from a sandboxed WebAssembly plugin.\",\"success\":true,\"outputs\":[{\"kind\":\"markdown\",\"title\":\"Widget\",\"body\":\"## Sandboxed Widget\\n- transport: wasm\\n- rendering target: web-friendly host\\n- note: this module executed in Wasmtime\"}],\"suggested_next_steps\":[\"Compare this with the native ui-panel plugin output.\"]}")
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
    i32.const 437
  )
)
