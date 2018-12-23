(module
  (func $select (result i32)
    i32.const 3
    i32.const 2
    i32.const 1
    select)
  (export "select" (func $select))
)