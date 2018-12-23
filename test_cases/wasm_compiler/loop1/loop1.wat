(module
  (type $t0 (func (result i32)))
  (func $loop1 (export "loop1") (type $t0)  (result i32)
    block $B1
        loop $L2
            loop $L3
                block $B4
                    br $B4
                end
                i32.const 0
                br_if $L2
                br $B1
            end
           
        end
    end
    i32.const 1))
