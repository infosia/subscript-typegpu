// module: m34-matrix-columns
// purpose: write a weighted sum of the components of each matrix column
// exercises: K10, K36, LB5
// questions: none

import { MutStorage } from "./typegpu";
import { Mat2x2f, Mat3x3f, Mat4x4f, Vec2f, Vec3f, Vec4f } from "./typegpu-types";

export class WordLayout {
  constructor(public words: MutStorage<f32>) {}
}

export function writeWords(res: WordLayout): void {
  let m2 = new Mat2x2f(new Vec2f(1.0, 2.0), new Vec2f(3.0, 4.0));
  m2.c0 = new Vec2f(5.0, 6.0);
  const weights2 = new Vec2f(1.0, 2.0);
  res.words[0] = m2.c0.dot(weights2);
  res.words[1] = m2.c1.dot(weights2);

  let m3 = new Mat3x3f(new Vec3f(11.0, 12.0, 13.0), new Vec3f(14.0, 15.0, 16.0), new Vec3f(17.0, 18.0, 19.0));
  m3.c1.y = 20.0;
  const weights3 = new Vec3f(1.0, 2.0, 4.0);
  res.words[2] = m3.c0.dot(weights3);
  res.words[3] = m3.c1.dot(weights3);
  res.words[4] = m3.c2.dot(weights3);

  let m4 = new Mat4x4f(
    new Vec4f(21.0, 22.0, 23.0, 24.0), new Vec4f(25.0, 26.0, 27.0, 28.0),
    new Vec4f(29.0, 30.0, 31.0, 32.0), new Vec4f(33.0, 34.0, 35.0, 36.0),
  );
  m4.c3.w += 4.0;
  const weights4 = new Vec4f(1.0, 2.0, 4.0, 8.0);
  res.words[5] = m4.c0.dot(weights4);
  res.words[6] = m4.c1.dot(weights4);
  res.words[7] = m4.c2.dot(weights4);
  res.words[8] = m4.c3.dot(weights4);
}

export function hostLayout(): WordLayout {
  return new WordLayout(new MutStorage<f32>([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
}
