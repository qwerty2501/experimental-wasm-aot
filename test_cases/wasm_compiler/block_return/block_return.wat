(module
  (type $t0 (func (result i32)))
  (func $block_return (export "block_return") (type $t0)  (result i32)
    block $B0 (result i32)
        block $Bb3(result i32)
            block $Bb4(result f32)
                i32.const 4
                br $B0
            end
            drop
            i32.const 0
        end
        drop
        i32.const 3
    end))
