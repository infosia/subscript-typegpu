import { UniformArray_SIZE } from "./sc10-uniform.typegpu";

class Uniform<T> {
  value: T;

  constructor(value: T) {
    this.value = value;
  }
}

@CStruct
class UniformArray {
  values: FixedArray<f32, 2>;
}

const marker: Uniform<UniformArray> = new Uniform<UniformArray>(new UniformArray());
