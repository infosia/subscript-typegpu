struct Digits {
  tens: u32,
  ones: u32,
}

fn tick() -> u32 {
  words[18u] += 1u; return words[18u];
}

@group(0u) @binding(0u) var<storage, read_write> words: array<u32>;

fn Digits_plus(self_: Digits, value: u32) -> u32 {
  return self_.tens * 10u + value;
}

fn nextVec() -> vec2<f32> {
  let v = f32(tick());
  return vec2<f32>(v, v);
}

fn edgeVec() -> vec2<f32> {
  return nextVec();
}

fn nextDigits() -> Digits {
  let _g_constructor_0 = tick();
  return Digits(_g_constructor_0, 0u);
}

fn pair(a: u32, b: u32) -> u32 {
  return a * 10u + b;
}

@compute @workgroup_size(1u, 1u, 1u)
fn orderKernel() {
  for (var k = 0u; k < 2u; k += 1u) {
    let base = k * 9u;
    words[18u] = 0u;
    let _g_operand_0 = tick();
    var _g_conditional_0: u32;
    if (k == 0u) {
      _g_conditional_0 = tick();
    } else {
      _g_conditional_0 = 0u;
    }
    words[base] = pair(_g_operand_0, _g_conditional_0);
    words[18u] = 0u;
    let _g_operand_1 = nextVec();
    words[base + 1u] = u32(step(edgeVec(), _g_operand_1).x);
    words[18u] = 0u;
    let _g_operand_2 = nextVec();
    words[base + 2u] = u32(smoothstep(edgeVec(), edgeVec(), _g_operand_2).x);
    words[18u] = 0u;
    var _g_logic_0: bool = k != 0u;
    if (_g_logic_0) {
      var _g_conditional_1: bool;
      if (k == 0u) {
        _g_conditional_1 = tick() > 0u;
      } else {
        _g_conditional_1 = false;
      }
      _g_logic_0 = _g_conditional_1;
    }
    let both = _g_logic_0;
    var _g_conditional_2: u32;
    if (both) {
      _g_conditional_2 = 1u;
    } else {
      _g_conditional_2 = 0u;
    }
    words[base + 3u] = pair(words[18u], _g_conditional_2);
    words[18u] = 0u;
    var _g_logic_1: bool = k != 0u;
    if (!_g_logic_1) {
      var _g_conditional_3: bool;
      if (k != 0u) {
        _g_conditional_3 = tick() > 0u;
      } else {
        _g_conditional_3 = false;
      }
      _g_logic_1 = _g_conditional_3;
    }
    let either = _g_logic_1;
    var _g_conditional_4: u32;
    if (either) {
      _g_conditional_4 = 1u;
    } else {
      _g_conditional_4 = 0u;
    }
    words[base + 4u] = pair(words[18u], _g_conditional_4);
    words[18u] = 0u;
    let _g_operand_3 = tick();
    var digits = Digits(tick(), _g_operand_3);
    words[base + 5u] = pair(digits.tens, digits.ones);
    words[18u] = 0u;
    let _g_operand_4 = nextDigits();
    var _g_conditional_5: u32;
    if (k == 0u) {
      _g_conditional_5 = tick();
    } else {
      _g_conditional_5 = 0u;
    }
    words[base + 6u] = Digits_plus(_g_operand_4, _g_conditional_5);
    words[18u] = 0u;
    var _g_conditional_6: u32;
    if (k == 0u) {
      _g_conditional_6 = tick();
    } else {
      _g_conditional_6 = 0u;
    }
    words[base + 7u] = pair(7u, _g_conditional_6) + pair(1u, 2u);
    words[18u] = 0u;
    let _g_operand_6 = tick();
    var _g_conditional_8: u32;
    if (k == 0u) {
      let _g_operand_5 = tick();
      var _g_conditional_7: u32;
      if (k == 0u) {
        _g_conditional_7 = tick();
      } else {
        _g_conditional_7 = 0u;
      }
      _g_conditional_8 = pair(_g_operand_5, _g_conditional_7);
    } else {
      _g_conditional_8 = 0u;
    }
    words[base + 8u] = pair(_g_operand_6, _g_conditional_8);
  }
}
