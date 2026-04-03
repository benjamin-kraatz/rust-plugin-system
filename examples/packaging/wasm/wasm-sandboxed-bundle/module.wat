(module
  (memory (export "memory") 2)
  (global $heap (mut i32) (i32.const 8192))

  ;; ── Response templates ───────────────────────────────────────────────
  ;; Offset 0 (len 417): echo response
  (data (i32.const 0)
    "{\"plugin_id\":\"wasm-sandboxed\",\"action_id\":\"echo\",\"title\":\"Echo from Sandbox\",\"summary\":\"Payload echoed from WASM sandbox.\",\"success\":true,\"outputs\":[{\"kind\":\"text\",\"title\":\"Echo\",\"body\":\"Your request was routed to the echo handler inside the WASM sandbox. The module parsed the action_id from the input JSON and dispatched accordingly.\"}],\"suggested_next_steps\":[\"Try the compute action.\",\"Try the validate action.\"]}")

  ;; Offset 512 (len 218): compute response prefix (before the number)
  (data (i32.const 512)
    "{\"plugin_id\":\"wasm-sandboxed\",\"action_id\":\"compute\",\"title\":\"Computation Result\",\"summary\":\"Computed factorial inside WASM sandbox.\",\"success\":true,\"outputs\":[{\"kind\":\"text\",\"title\":\"Factorial\",\"body\":\"factorial(10) = ")

  ;; Offset 768 (len 161): compute response suffix (after the number)
  (data (i32.const 768)
    ". Computed entirely inside the WebAssembly sandbox using iterative multiplication.\"}],\"suggested_next_steps\":[\"Try the echo action.\",\"Try the validate action.\"]}")

  ;; Offset 1024 (len 369): validate-ok response
  (data (i32.const 1024)
    "{\"plugin_id\":\"wasm-sandboxed\",\"action_id\":\"validate\",\"title\":\"Validation Passed\",\"summary\":\"Input payload structure validated.\",\"success\":true,\"outputs\":[{\"kind\":\"text\",\"title\":\"Valid\",\"body\":\"The input payload was received and validated. The WASM module confirmed the payload contains data.\"}],\"suggested_next_steps\":[\"Try the echo action.\",\"Try the compute action.\"]}")

  ;; Offset 1536 (len 353): validate-fail response
  (data (i32.const 1536)
    "{\"plugin_id\":\"wasm-sandboxed\",\"action_id\":\"validate\",\"title\":\"Validation Note\",\"summary\":\"Input payload was empty or missing.\",\"success\":true,\"outputs\":[{\"kind\":\"text\",\"title\":\"Empty Payload\",\"body\":\"The input payload was empty or null. Provide a non-empty JSON object for validation.\"}],\"suggested_next_steps\":[\"Retry with a payload containing data.\"]}")

  ;; Offset 2048 (len 347): unknown-action response
  (data (i32.const 2048)
    "{\"plugin_id\":\"wasm-sandboxed\",\"action_id\":\"unknown\",\"title\":\"Unknown Action\",\"summary\":\"The requested action was not recognized.\",\"success\":false,\"outputs\":[{\"kind\":\"text\",\"title\":\"Error\",\"body\":\"Supported actions: echo, compute, validate. Check the action_id in your request.\"}],\"suggested_next_steps\":[\"Use action: echo, compute, or validate.\"]}")

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

  ;; ── Iterative factorial ──────────────────────────────────────────────
  (func $factorial (param $n i32) (result i32)
    (local $result i32)
    (local $i i32)
    i32.const 1
    local.set $result
    i32.const 2
    local.set $i
    (block $done
      (loop $loop
        local.get $i
        local.get $n
        i32.gt_s
        br_if $done
        local.get $result
        local.get $i
        i32.mul
        local.set $result
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        br $loop
      )
    )
    local.get $result
  )

  ;; ── Integer-to-ASCII (writes decimal digits, returns byte count) ────
  (func $itoa (param $n i32) (param $buf i32) (result i32)
    (local $tmp i32)
    (local $digits i32)
    (local $i i32)
    (local $start i32)
    (local $end i32)
    (local $swap i32)

    ;; Count digits and write them in reverse
    local.get $n
    local.set $tmp
    i32.const 0
    local.set $digits

    (block $done
      (loop $count_loop
        local.get $buf
        local.get $digits
        i32.add
        local.get $tmp
        i32.const 10
        i32.rem_u
        i32.const 48  ;; ASCII '0'
        i32.add
        i32.store8

        local.get $digits
        i32.const 1
        i32.add
        local.set $digits

        local.get $tmp
        i32.const 10
        i32.div_u
        local.set $tmp

        local.get $tmp
        i32.const 0
        i32.gt_u
        br_if $count_loop
      )
    )

    ;; Reverse the digits in-place
    local.get $buf
    local.set $start
    local.get $buf
    local.get $digits
    i32.add
    i32.const 1
    i32.sub
    local.set $end

    (block $rev_done
      (loop $rev_loop
        local.get $start
        local.get $end
        i32.ge_u
        br_if $rev_done

        ;; Swap bytes at $start and $end
        local.get $start
        i32.load8_u
        local.set $swap
        local.get $start
        local.get $end
        i32.load8_u
        i32.store8
        local.get $end
        local.get $swap
        i32.store8

        local.get $start
        i32.const 1
        i32.add
        local.set $start
        local.get $end
        i32.const 1
        i32.sub
        local.set $end
        br $rev_loop
      )
    )

    local.get $digits
  )

  ;; ── Memory copy ──────────────────────────────────────────────────────
  (func $memcpy (param $dst i32) (param $src i32) (param $len i32)
    (local $i i32)
    i32.const 0
    local.set $i
    (block $done
      (loop $loop
        local.get $i
        local.get $len
        i32.ge_u
        br_if $done
        local.get $dst
        local.get $i
        i32.add
        local.get $src
        local.get $i
        i32.add
        i32.load8_u
        i32.store8
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        br $loop
      )
    )
  )

  ;; ── Find "action_id":" in the input and return the byte after it ────
  ;; Returns the first character of the action_id value, or 0 if not found.
  (func $find_action_char (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $end i32)
    i32.const 0
    local.set $i
    ;; We need at least 14 bytes for the pattern + 1 for action char
    local.get $len
    i32.const 15
    i32.sub
    local.set $end

    (block $not_found
      (loop $scan
        local.get $i
        local.get $end
        i32.gt_s
        br_if $not_found

        ;; Check for "action_id":" (13 bytes: 0x22 61 63 74 69 6f 6e 5f 69 64 22 3a 22)
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
            ;; Return the first char of the action_id value (at offset +13)
            (return (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 13)))))
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

  ;; ── Check if payload is non-empty (looks for "payload":{} or null) ──
  (func $payload_has_content (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $end i32)
    i32.const 0
    local.set $i
    local.get $len
    i32.const 12
    i32.sub
    local.set $end

    (block $not_found
      (loop $scan
        local.get $i
        local.get $end
        i32.gt_s
        br_if $not_found

        ;; Check for "payload": (10 bytes: 22 70 61 79 6c 6f 61 64 22 3a)
        (if (i32.and
              (i32.and
                (i32.eq (i32.load8_u (i32.add (local.get $ptr) (local.get $i)))            (i32.const 34))   ;; "
                (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 1))))  (i32.const 112))  ;; p
              )
              (i32.and
                (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 7))))  (i32.const 100))  ;; d
                (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 9))))  (i32.const 58))   ;; :
              )
            )
          (then
            ;; Found "payload":, now check char after colon
            ;; Skip whitespace after colon
            (local.set $i (i32.add (local.get $i) (i32.const 10)))
            ;; Check if it's "null" or "{}" (empty)
            (if (i32.eq (i32.load8_u (i32.add (local.get $ptr) (local.get $i))) (i32.const 110)) ;; 'n' for null
              (then (return (i32.const 0)))
            )
            (if (i32.and
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (local.get $i))) (i32.const 123)) ;; {
                  (i32.eq (i32.load8_u (i32.add (local.get $ptr) (i32.add (local.get $i) (i32.const 1)))) (i32.const 125)) ;; }
                )
              (then (return (i32.const 0)))
            )
            ;; Has content
            (return (i32.const 1))
          )
        )

        local.get $i
        i32.const 1
        i32.add
        local.set $i
        br $scan
      )
    )
    ;; payload key not found → treat as empty
    i32.const 0
  )

  ;; ── Main entry point ─────────────────────────────────────────────────
  (func (export "invoke_json") (param $in_ptr i32) (param $in_len i32) (result i32 i32)
    (local $action_char i32)
    (local $out_ptr i32)
    (local $out_len i32)
    (local $num_len i32)

    ;; Determine action from first char of action_id
    local.get $in_ptr
    local.get $in_len
    call $find_action_char
    local.set $action_char

    ;; Route: 'e'=echo, 'c'=compute, 'v'=validate
    (if (i32.eq (local.get $action_char) (i32.const 101))  ;; 'e' → echo
      (then
        (return (i32.const 0) (i32.const 417))
      )
    )

    (if (i32.eq (local.get $action_char) (i32.const 99))   ;; 'c' → compute
      (then
        ;; Build response dynamically: prefix + factorial(10) + suffix
        ;; Allocate in heap area starting at current $heap
        global.get $heap
        local.set $out_ptr

        ;; Copy prefix (218 bytes from offset 512)
        local.get $out_ptr
        i32.const 512
        i32.const 218
        call $memcpy

        ;; Write factorial(10) as ASCII digits
        i32.const 10
        call $factorial
        local.get $out_ptr
        i32.const 218
        i32.add
        call $itoa
        local.set $num_len

        ;; Copy suffix (161 bytes from offset 768)
        local.get $out_ptr
        i32.const 218
        i32.add
        local.get $num_len
        i32.add
        i32.const 768
        i32.const 161
        call $memcpy

        ;; Total length = 218 + num_len + 161
        i32.const 218
        local.get $num_len
        i32.add
        i32.const 161
        i32.add
        local.set $out_len

        ;; Advance heap past the response
        global.get $heap
        local.get $out_len
        i32.add
        global.set $heap

        local.get $out_ptr
        local.get $out_len
        return
      )
    )

    (if (i32.eq (local.get $action_char) (i32.const 118))  ;; 'v' → validate
      (then
        ;; Check if payload has content
        (if (i32.eqz (call $payload_has_content (local.get $in_ptr) (local.get $in_len)))
          (then
            (return (i32.const 1536) (i32.const 353))  ;; validate-fail
          )
        )
        (return (i32.const 1024) (i32.const 369))  ;; validate-ok
      )
    )

    ;; Fallback: unknown action
    i32.const 2048
    i32.const 347
  )
)
