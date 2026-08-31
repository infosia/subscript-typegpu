import {
  Vec2f,
  Vec3f,
  clamp,
  fract,
  mix,
  sign,
} from "./typegpu-types";

function colorMax(a: f32, b: f32): f32 {
  return a > b ? a : b;
}

function colorMin(a: f32, b: f32): f32 {
  return a < b ? a : b;
}

function colorAbs(value: f32): f32 {
  return value < 0.0 ? -value : value;
}

function colorSqrt(value: f32): f32 {
  return new Vec2f(value, value).sqrt().x;
}

function colorPow(value: f32, exponent: f32): f32 {
  return new Vec2f(value, value).pow(new Vec2f(exponent, exponent)).x;
}

function colorCbrt(value: f32): f32 {
  return sign(value) * colorPow(colorAbs(value), 0.3333333333333333);
}

// Converts linear-light RGB to the componentwise sRGB transfer curve.
export function linearToSrgb(c: Vec3f): Vec3f {
  const low: Vec3f = c.scale(12.92);
  const high: Vec3f = new Vec3f(
    1.055 * colorPow(c.x, 1.0 / 2.4) - 0.055,
    1.055 * colorPow(c.y, 1.0 / 2.4) - 0.055,
    1.055 * colorPow(c.z, 1.0 / 2.4) - 0.055,
  );
  return high.select(low, c.le(new Vec3f(0.0031308, 0.0031308, 0.0031308)));
}

// Converts sRGB to linear-light RGB with the inverse componentwise transfer curve.
export function srgbToLinear(c: Vec3f): Vec3f {
  const low: Vec3f = c.scale(1.0 / 12.92);
  const shifted: Vec3f = c.add(new Vec3f(0.055, 0.055, 0.055)).scale(1.0 / 1.055);
  const high: Vec3f = new Vec3f(
    colorPow(shifted.x, 2.4),
    colorPow(shifted.y, 2.4),
    colorPow(shifted.z, 2.4),
  );
  return high.select(low, c.le(new Vec3f(0.04045, 0.04045, 0.04045)));
}

// HSV uses hue turns in c.x and saturation/value in c.yz.
export function hsvToRgb(c: Vec3f): Vec3f {
  const sectors = new Vec3f(
    fract(c.x),
    fract(c.x + 0.6666666666666666),
    fract(c.x + 0.3333333333333333),
  ).scale(6.0).sub(new Vec3f(3.0, 3.0, 3.0)).abs();
  const primary = sectors.sub(new Vec3f(1.0, 1.0, 1.0)).clamp(
    new Vec3f(0.0, 0.0, 0.0),
    new Vec3f(1.0, 1.0, 1.0),
  );
  return new Vec3f(1.0, 1.0, 1.0).mix(primary, c.y).scale(c.z);
}

// Returns hue in turns and the usual saturation/value pair.
export function rgbToHsv(c: Vec3f): Vec3f {
  const maximum: f32 = colorMax(c.x, colorMax(c.y, c.z));
  const minimum: f32 = colorMin(c.x, colorMin(c.y, c.z));
  const chroma: f32 = maximum - minimum;
  let hue: f32 = 0.0;
  if (chroma > 0.0) {
    if (maximum === c.x) {
      hue = (c.y - c.z) / chroma;
      if (hue < 0.0) hue += 6.0;
    } else if (maximum === c.y) {
      hue = (c.z - c.x) / chroma + 2.0;
    } else {
      hue = (c.x - c.y) / chroma + 4.0;
    }
    hue /= 6.0;
  }
  const saturation: f32 = maximum > 0.0 ? chroma / maximum : 0.0;
  return new Vec3f(hue, saturation, maximum);
}

// Bjorn Ottosson's linear-sRGB to Oklab matrix pair, with signed cube roots.
export function linearRgbToOklab(c: Vec3f): Vec3f {
  const l: f32 = 0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z;
  const m: f32 = 0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z;
  const s: f32 = 0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z;
  const lRoot: f32 = colorCbrt(l);
  const mRoot: f32 = colorCbrt(m);
  const sRoot: f32 = colorCbrt(s);
  return new Vec3f(
    0.2104542553 * lRoot + 0.7936177850 * mRoot - 0.0040720468 * sRoot,
    1.9779984951 * lRoot - 2.4285922050 * mRoot + 0.4505937099 * sRoot,
    0.0259040371 * lRoot + 0.7827717662 * mRoot - 0.8086757660 * sRoot,
  );
}

// Bjorn Ottosson's inverse Oklab matrices, cubing the intermediate LMS values.
export function oklabToLinearRgb(c: Vec3f): Vec3f {
  const lRoot: f32 = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
  const mRoot: f32 = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
  const sRoot: f32 = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;
  const l: f32 = lRoot * lRoot * lRoot;
  const m: f32 = mRoot * mRoot * mRoot;
  const s: f32 = sRoot * sRoot * sRoot;
  return new Vec3f(
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
  );
}

// Converts display-referred sRGB to Oklab through linear light.
export function rgbToOklab(c: Vec3f): Vec3f {
  return linearRgbToOklab(srgbToLinear(c));
}

function computeMaxSaturation(a: f32, b: f32): f32 {
  let k0: f32 = 0.0;
  let k1: f32 = 0.0;
  let k2: f32 = 0.0;
  let k3: f32 = 0.0;
  let k4: f32 = 0.0;
  let wl: f32 = 0.0;
  let wm: f32 = 0.0;
  let ws: f32 = 0.0;
  if (-1.88170328 * a - 0.80936493 * b > 1.0) {
    k0 = 1.19086277;
    k1 = 1.76576728;
    k2 = 0.59662641;
    k3 = 0.75515197;
    k4 = 0.56771245;
    wl = 4.0767416621;
    wm = -3.3077115913;
    ws = 0.2309699292;
  } else if (1.81444104 * a - 1.19445276 * b > 1.0) {
    k0 = 0.73956515;
    k1 = -0.45954404;
    k2 = 0.08285427;
    k3 = 0.12541070;
    k4 = 0.14503204;
    wl = -1.2684380046;
    wm = 2.6097574011;
    ws = -0.3413193965;
  } else {
    k0 = 1.35733652;
    k1 = -0.00915799;
    k2 = -1.15130210;
    k3 = -0.50559606;
    k4 = 0.00692167;
    wl = -0.0041960863;
    wm = -0.7034186147;
    ws = 1.7076147010;
  }

  let saturation: f32 = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;
  const kl: f32 = 0.3963377774 * a + 0.2158037573 * b;
  const km: f32 = -0.1055613458 * a - 0.0638541728 * b;
  const ks: f32 = -0.0894841775 * a - 1.2914855480 * b;
  const lRoot: f32 = 1.0 + saturation * kl;
  const mRoot: f32 = 1.0 + saturation * km;
  const sRoot: f32 = 1.0 + saturation * ks;
  const l: f32 = lRoot * lRoot * lRoot;
  const m: f32 = mRoot * mRoot * mRoot;
  const s: f32 = sRoot * sRoot * sRoot;
  const l1: f32 = 3.0 * kl * lRoot * lRoot;
  const m1: f32 = 3.0 * km * mRoot * mRoot;
  const s1: f32 = 3.0 * ks * sRoot * sRoot;
  const l2: f32 = 6.0 * kl * kl * lRoot;
  const m2: f32 = 6.0 * km * km * mRoot;
  const s2: f32 = 6.0 * ks * ks * sRoot;
  const f: f32 = wl * l + wm * m + ws * s;
  const f1: f32 = wl * l1 + wm * m1 + ws * s1;
  const f2: f32 = wl * l2 + wm * m2 + ws * s2;
  saturation -= f * f1 / (f1 * f1 - 0.5 * f * f2);
  return saturation;
}

function findCusp(a: f32, b: f32): Vec2f {
  const saturation: f32 = computeMaxSaturation(a, b);
  const rgb: Vec3f = oklabToLinearRgb(new Vec3f(1.0, saturation * a, saturation * b));
  const lightness: f32 = colorCbrt(1.0 / colorMax(rgb.x, colorMax(rgb.y, rgb.z)));
  return new Vec2f(lightness, lightness * saturation);
}

function halleyDelta(value: f32, first: f32, second: f32): f32 {
  const denominator: f32 = first * first - 0.5 * value * second;
  const ratio: f32 = first / denominator;
  return ratio >= 0.0 ? -value * ratio : 1000000.0;
}

function findGamutIntersection(
  a: f32,
  b: f32,
  lightness: f32,
  chroma: f32,
  lightness0: f32,
  cusp: Vec2f,
): f32 {
  let amount: f32 = 0.0;
  if ((lightness - lightness0) * cusp.y - (cusp.x - lightness0) * chroma <= 0.0) {
    amount = cusp.y * lightness0
      / (chroma * cusp.x + cusp.y * (lightness0 - lightness));
  } else {
    amount = cusp.y * (lightness0 - 1.0)
      / (chroma * (cusp.x - 1.0) + cusp.y * (lightness0 - lightness));
    const deltaLightness: f32 = lightness - lightness0;
    const deltaChroma: f32 = chroma;
    const kl: f32 = 0.3963377774 * a + 0.2158037573 * b;
    const km: f32 = -0.1055613458 * a - 0.0638541728 * b;
    const ks: f32 = -0.0894841775 * a - 1.2914855480 * b;
    const lRoot: f32 = lightness0 * (1.0 - amount) + amount * lightness
      + amount * chroma * kl;
    const mRoot: f32 = lightness0 * (1.0 - amount) + amount * lightness
      + amount * chroma * km;
    const sRoot: f32 = lightness0 * (1.0 - amount) + amount * lightness
      + amount * chroma * ks;
    const lDelta: f32 = deltaLightness + deltaChroma * kl;
    const mDelta: f32 = deltaLightness + deltaChroma * km;
    const sDelta: f32 = deltaLightness + deltaChroma * ks;
    const l: f32 = lRoot * lRoot * lRoot;
    const m: f32 = mRoot * mRoot * mRoot;
    const s: f32 = sRoot * sRoot * sRoot;
    const l1: f32 = 3.0 * lDelta * lRoot * lRoot;
    const m1: f32 = 3.0 * mDelta * mRoot * mRoot;
    const s1: f32 = 3.0 * sDelta * sRoot * sRoot;
    const l2: f32 = 6.0 * lDelta * lDelta * lRoot;
    const m2: f32 = 6.0 * mDelta * mDelta * mRoot;
    const s2: f32 = 6.0 * sDelta * sDelta * sRoot;
    const red: f32 = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s - 1.0;
    const red1: f32 = 4.0767416621 * l1 - 3.3077115913 * m1 + 0.2309699292 * s1;
    const red2: f32 = 4.0767416621 * l2 - 3.3077115913 * m2 + 0.2309699292 * s2;
    const green: f32 = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s - 1.0;
    const green1: f32 = -1.2684380046 * l1 + 2.6097574011 * m1 - 0.3413193965 * s1;
    const green2: f32 = -1.2684380046 * l2 + 2.6097574011 * m2 - 0.3413193965 * s2;
    const blue: f32 = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s - 1.0;
    const blue1: f32 = -0.0041960863 * l1 - 0.7034186147 * m1 + 1.7076147010 * s1;
    const blue2: f32 = -0.0041960863 * l2 - 0.7034186147 * m2 + 1.7076147010 * s2;
    amount += colorMin(
      halleyDelta(red, red1, red2),
      colorMin(halleyDelta(green, green1, green2), halleyDelta(blue, blue1, blue2)),
    );
  }
  return amount;
}

// Clips Oklab toward an adaptive lightness anchor while preserving hue.
export function oklabGamutClipAdaptiveL05(lab: Vec3f, alpha: f32): Vec3f {
  const chroma: f32 = colorMax(0.00001, lab.yz.length());
  const a: f32 = lab.y / chroma;
  const b: f32 = lab.z / chroma;
  const lightnessDelta: f32 = lab.x - 0.5;
  const absoluteDelta: f32 = colorAbs(lightnessDelta);
  const e1: f32 = 0.5 + absoluteDelta + alpha * chroma;
  const lightness0: f32 = 0.5 * (
    1.0 + sign(lightnessDelta) * (
      e1 - colorSqrt(colorMax(0.0, e1 * e1 - 2.0 * absoluteDelta))
    )
  );
  const amount: f32 = clamp(
    findGamutIntersection(a, b, lab.x, chroma, lightness0, findCusp(a, b)),
    0.0,
    1.0,
  );
  const clippedLightness: f32 = mix(lightness0, lab.x, amount);
  const clippedChroma: f32 = amount * chroma;
  return new Vec3f(clippedLightness, clippedChroma * a, clippedChroma * b);
}

// Clips Oklab adaptively before applying the display-referred sRGB transfer curve.
export function oklabToRgb(c: Vec3f): Vec3f {
  return linearToSrgb(oklabToLinearRgb(oklabGamutClipAdaptiveL05(c, 0.2)));
}
