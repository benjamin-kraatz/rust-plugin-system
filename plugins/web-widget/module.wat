(module
  (memory (export "memory") 2)
  (global $heap (mut i32) (i32.const 8192))

  ;; ── Response templates ───────────────────────────────────────────────
  ;; Offset 0 (len 422): render-badge response
  (data (i32.const 0)
    "{\"plugin_id\":\"web-widget\",\"action_id\":\"render-badge\",\"title\":\"Badge Rendered\",\"summary\":\"Generated a text badge in the WASM sandbox.\",\"success\":true,\"outputs\":[{\"kind\":\"markdown\",\"title\":\"Badge\",\"body\":\"+===================+\\n|  STATUS: OK       |\\n|  Plugin: active   |\\n|  Sandbox: wasm    |\\n+===================+\\nRendered inside WebAssembly sandbox.\"}],\"suggested_next_steps\":[\"Try render-table or render-progress.\"]}")

  ;; Offset 512 (len 406): render-table response
  (data (i32.const 512)
    "{\"plugin_id\":\"web-widget\",\"action_id\":\"render-table\",\"title\":\"Table Rendered\",\"summary\":\"Generated a text table in the WASM sandbox.\",\"success\":true,\"outputs\":[{\"kind\":\"markdown\",\"title\":\"Data Table\",\"body\":\"| Metric   | Value |\\n|----------|-------|\\n| CPU      | 42%   |\\n| Memory   | 68%   |\\n| Uptime   | 99.9% |\\n| Requests | 1.2k  |\"}],\"suggested_next_steps\":[\"Try render-badge or render-progress.\"]}")

  ;; Offset 1024 (len 361): render-progress response
  (data (i32.const 1024)
    "{\"plugin_id\":\"web-widget\",\"action_id\":\"render-progress\",\"title\":\"Progress Bar\",\"summary\":\"Generated a progress bar in the WASM sandbox.\",\"success\":true,\"outputs\":[{\"kind\":\"markdown\",\"title\":\"Progress\",\"body\":\"Task: Deployment\\n[==============------] 70%\\nStatus: In Progress\\nETA: 3 min remaining\"}],\"suggested_next_steps\":[\"Try render-badge or render-table.\"]}")

  ;; Offset 1536 (len 353): unknown action response
  (data (i32.const 1536)
    "{\"plugin_id\":\"web-widget\",\"action_id\":\"unknown\",\"title\":\"Unknown Widget\",\"summary\":\"The requested widget action was not recognized.\",\"success\":false,\"outputs\":[{\"kind\":\"text\",\"title\":\"Error\",\"body\":\"Supported actions: render-badge, render-table, render-progress.\"}],\"suggested_next_steps\":[\"Use action: render-badge, render-table, or render-progress.\"]}")

  ;; ── Bump allocator ───────────────────────────────────────────────────
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

  ;; ── Find action_id value and return the character at offset +7 ──────
  ;; For render-badge/render-table/render-progress, all start with "render-"
  ;; so we distinguish by the 8th character: 'b', 't', or 'p'.
  (func $find_action_discriminator (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $end i32)
    i32.const 0
    local.set $i
    local.get $len
    i32.const 22  ;; need at least 13 + 8 + 1 bytes
    i32.sub
    local.set $end

    (block $not_found
      (loop $scan
        local.get $i
        local.get $end
        i32.gt_s
        br_if $not_found

        ;; Check for "action_id":" (match key bytes: " a c t ... d " : ")
        (if (i32.and
              (i32.and
                (i32.and
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (local.get $i)))            (i32.const 34))   ;; "
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 1))))  (i32.const 97))   ;; a
                )
                (i32.and
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 2))))  (i32.const 99))   ;; c
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 3))))  (i32.const 116))  ;; t
                )
              )
              (i32.and
                (i32.and
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 4))))  (i32.const 105))  ;; i
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 5))))  (i32.const 111))  ;; o
                )
                (i32.and
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 6))))  (i32.const 110))  ;; n
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 12)))) (i32.const 34))   ;; closing "
                )
              )
            )
          (then
            ;; Return char at offset +13+7 = +20 (the discriminating char after "render-")
            (return (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 20)))))
          )
        )

        local.get $i
        i32.const 1
        i32.add
        local.set $i
        br $scan
      )
    )
    i32.const 0
  )

  ;; ── Main entry point ─────────────────────────────────────────────────
  (func (export "invoke_json") (param $in_ptr i32) (param $in_len i32) (result i32 i32)
    (local $disc i32)

    local.get $in_ptr
    local.get $in_len
    call $find_action_discriminator
    local.set $disc

    ;; Route: 'b'=badge(98), 't'=table(116), 'p'=progress(112)
    (if (i32.eq (local.get $disc) (i32.const 98))    ;; 'b' → render-badge
      (then (return (i32.const 0) (i32.const 422)))
    )
    (if (i32.eq (local.get $disc) (i32.const 116))   ;; 't' → render-table
      (then (return (i32.const 512) (i32.const 406)))
    )
    (if (i32.eq (local.get $disc) (i32.const 112))   ;; 'p' → render-progress
      (then (return (i32.const 1024) (i32.const 361)))
    )

    ;; Fallback: unknown action
    i32.const 1536
    i32.const 353
  )
)
