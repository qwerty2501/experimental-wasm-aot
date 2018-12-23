(module
  (type $t0 (func (param i32) (result i32)))
  (func $pow (export "pow") (type $t0) (param $p0 i32) (result i32)
    (local $l0 i32) (local $l1 i32) (local $l2 i32) (local $l3 i32)
    i32.const 2
    set_local $l0
    block $B0
      block $B1
        get_local $p0
        i32.const 2
        i32.lt_u
        br_if $B1
        i32.const 2
        set_local $l0
        i32.const 1
        set_local $l1
        loop $L2
          get_local $l0
          i32.const 1
          get_local $p0
          i32.const 1
          i32.and
          select
          get_local $l1
          i32.mul
          set_local $l1
          get_local $p0
          i32.const 3
          i32.gt_u
          set_local $l2
          get_local $l0
          get_local $l0
          i32.mul
          set_local $l0
          get_local $p0
          i32.const 1
          i32.shr_u
          tee_local $l3
          set_local $p0
          get_local $l2
          br_if $L2
          br $B0
        end
      end
      get_local $p0
      set_local $l3
      i32.const 1
      set_local $l1
    end
    get_local $l0
    i32.const 1
    get_local $l3
    i32.const 1
    i32.eq
    select
    get_local $l1
    i32.mul))
