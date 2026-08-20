
(component
  (core module $module
    (func (export "run") (param i32) (result i32)
      local.get 0
      i32.const 1
      i32.add))
  (core instance $instance (instantiate $module))
  (func (export "run") (param "input" s32) (result s32)
    (canon lift (core func $instance "run"))))
