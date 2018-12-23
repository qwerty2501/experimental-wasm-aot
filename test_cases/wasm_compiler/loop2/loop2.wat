(module
  (type $t0 (func))
  (type $t1 (func (result i32)))
  (func $__wasm_call_ctors (type $t0))
  (func $loop2 (export "loop2") (type $t1) (result i32)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32) (local $l4 i32) (local $l5 i32) (local $l6 i32) (local $l7 i32) (local $l8 i32) (local $l9 i32) (local $l10 i32) (local $l11 i32) (local $l12 i32) (local $l13 i32) (local $l14 i32) (local $l15 i32) (local $l16 i32) (local $l17 i32) (local $l18 i32) (local $l19 i32) (local $l20 i32) (local $l21 i32) (local $l22 i32) (local $l23 i32) (local $l24 i32) (local $l25 i32)
    get_global $g0
    set_local $l0
    i32.const 16
    set_local $l1
    get_local $l0
    get_local $l1
    i32.sub
    set_local $l2
    i32.const 0
    set_local $l3
    get_local $l2
    get_local $l3
    i32.store offset=12
    block $B0
      loop $L1
        i32.const 100
        set_local $l4
        get_local $l2
        i32.load offset=12
        set_local $l5
        get_local $l5
        set_local $l6
        get_local $l4
        set_local $l7
        get_local $l6
        get_local $l7
        i32.lt_s
        set_local $l8
        get_local $l8
        set_local $l9
        get_local $l9
        i32.eqz
        br_if $B0
        i32.const 3
        set_local $l10
        get_local $l2
        i32.load offset=12
        set_local $l11
        get_local $l11
        set_local $l12
        get_local $l10
        set_local $l13
        get_local $l12
        get_local $l13
        i32.eq
        set_local $l14
        get_local $l14
        set_local $l15
        block $B2
          block $B3
            get_local $l15
            i32.eqz
            br_if $B3
            br $B2
          end
          i32.const 5
          set_local $l16
          get_local $l2
          i32.load offset=12
          set_local $l17
          get_local $l17
          set_local $l18
          get_local $l16
          set_local $l19
          get_local $l18
          get_local $l19
          i32.eq
          set_local $l20
          get_local $l20
          set_local $l21
          block $B4
            get_local $l21
            i32.eqz
            br_if $B4
          end
        end
        get_local $l2
        i32.load offset=12
        set_local $l22
        i32.const 1
        set_local $l23
        get_local $l22
        get_local $l23
        i32.add
        set_local $l24
        get_local $l2
        get_local $l24
        i32.store offset=12
        br $L1
      end
    end
    i32.const 1
    set_local $l25
    get_local $l25
    return)
  (table $T0 1 1 anyfunc)
  (memory $memory (export "memory") 2)
  (global $g0 (mut i32) (i32.const 66560))
  (global $__heap_base (export "__heap_base") i32 (i32.const 66560))
  (global $__data_end (export "__data_end") i32 (i32.const 1024)))
