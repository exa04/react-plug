export interface Range {
  getMin: () => number,
  getMax: () => number,
  clamp: (n: number) => number,
  normalize: (n: number) => number,
  unnormalize: (n: number) => number,
}

export class LinearRange implements Range {
  min: number; max: number;
  getMin = () => this.min;
  getMax = () => this.max;

  clamp = (n: number) => clamp(n, this.min, this.max);
  normalize = (n: number) => (this.clamp(n) - this.min) / (this.max - this.min);
  unnormalize = (n: number) => this.clamp(n * (this.max - this.min) + this.min);

  constructor(min: number, max: number) {
    this.min = min;
    this.max = max;
  }
}

export class SkewedRange implements Range {
  min: number; max: number; factor: number;
  getMin = () => this.min;
  getMax = () => this.max;

  clamp = (n: number) => clamp(n, this.min, this.max);
  normalize = (n: number) => Math.pow((this.clamp(n) - this.min) / (this.max - this.min), this.factor);
  unnormalize = (n: number) => Math.pow(n, 1/this.factor) * (this.max - this.min) + this.min;

  constructor(min: number, max: number, factor: number) {
    this.min = min;
    this.max = max;
    this.factor = factor;
  }
}

export class SymmetricalSkewedRange implements Range {
  min: number; max: number; factor: number; center: number;
  getMin = () => this.min;
  getMax = () => this.max;

  clamp = (n: number) => clamp(n, this.min, this.max);
  normalize = (n: number) => {
    const unscaled_proportion = (this.clamp(n) - this.min) / (this.max - this.min);
    const center_proportion = (this.center - this.min) / (this.max - this.min);
    if (unscaled_proportion > center_proportion) {
      const scaled_proportion = (unscaled_proportion - center_proportion)
        * (1 / (1.0 - center_proportion));
      return (Math.pow(scaled_proportion, this.factor) * 0.5) + 0.5
    } else {
      const inverted_scaled_proportion =
        (center_proportion - unscaled_proportion) * (1 / center_proportion);
      return (1.0 - Math.pow(inverted_scaled_proportion, this.factor)) * 0.5
    }
  };
  unnormalize = (n: number) => {
    // Reconstructing the subranges works the same as with the normal skewed ranges
    const center_proportion = (this.center - this.min) / (this.max - this.min);
    let skewed_proportion;
    if (n > 0.5) {
      const scaled_proportion = (n - 0.5) * 2.0;
      skewed_proportion = (Math.pow(scaled_proportion, 1 / this.factor) * (1.0 - center_proportion)) + center_proportion
    } else {
      const inverted_scaled_proportion = (0.5 - n) * 2.0;
      skewed_proportion = (1.0 - Math.pow(inverted_scaled_proportion, 1 / this.factor)) * center_proportion
    };

    return skewed_proportion * (this.max - this.min) + this.min
  };

  constructor(min: number, max: number, factor: number, center: number) {
    this.min = min;
    this.max = max;
    this.factor = factor;
    this.center = center;
  }
}

export class ReversedRange implements Range {
  range: Range;
  getMin = () => this.range.getMax();
  getMax = () => this.range.getMin();

  clamp = (n: number) => this.range.clamp(n);
  normalize = (n: number) => 1 - this.range.normalize(n);
  unnormalize = (n: number) => this.range.unnormalize(1 - n);

  constructor(range: Range) {
    this.range = range;
  }
}

const clamp = (n: number, min: number, max: number): number => {
  if(n < min) return min;
  if(n > max) return max;
  return n;
}