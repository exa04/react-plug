export interface FloatRange {
  getMin: () => number,
  getMax: () => number,
  clamp: (n: number) => number,
  normalize: (n: number) => number,
  unnormalize: (n: number) => number,
  snap_to_step: (value: number, stepSize: number) => number,
  previousStep: (from: number, stepSize?: number, finer?: boolean) => number,
  nextStep: (from: number, stepSize?: number, finer?: boolean) => number,
}

export class LinearRange implements FloatRange {
  min: number;
  max: number;
  getMin = () => this.min;
  getMax = () => this.max;

  clamp = (n: number) => clamp(n, this.min, this.max);
  normalize = (n: number) => (this.clamp(n) - this.min) / (this.max - this.min);
  unnormalize = (n: number) => this.clamp(n * (this.max - this.min) + this.min);

  previousStep = (from: number, stepSize?: number, finer?: boolean) => {
    const normalized_naive_step_size = finer ? 0.005 : 0.02;
    const naive_step = this.unnormalize(this.normalize(from) - normalized_naive_step_size);

    let result;
    if (stepSize === undefined) {
      result = naive_step
    } else if (Math.abs(naive_step - from) > stepSize) {
      result = this.snap_to_step(naive_step, stepSize)
    } else {
      result = from - stepSize
    }

    return clamp(result, this.min, this.max);
  };

  nextStep = (from: number, stepSize?: number, finer?: boolean) => {
    const normalized_naive_step_size = finer ? 0.005 : 0.02;
    const naive_step = this.unnormalize(this.normalize(from) + normalized_naive_step_size);

    let result;
    if (stepSize === undefined) {
      result = naive_step
    } else if (Math.abs(naive_step - from) > stepSize) {
      result = this.snap_to_step(naive_step, stepSize)
    } else {
      result = from + stepSize
    }

    return clamp(result, this.min, this.max);
  };

  snap_to_step = (value: number, stepSize: number) => clamp(Math.round(value / stepSize) * stepSize, this.min, this.max);

  constructor(min: number, max: number) {
    this.min = min;
    this.max = max;
  }
}

/*export class SkewedRange implements FloatRange {
  min: number;
  max: number;
  factor: number;
  getMin = () => this.min;
  getMax = () => this.max;

  clamp = (n: number) => clamp(n, this.min, this.max);
  normalize = (n: number) => Math.pow((this.clamp(n) - this.min) / (this.max - this.min), this.factor);
  unnormalize = (n: number) => Math.pow(n, 1 / this.factor) * (this.max - this.min) + this.min;

  constructor(min: number, max: number, factor: number) {
    this.min = min;
    this.max = max;
    this.factor = factor;
  }
}

export class SymmetricalSkewedRange implements FloatRange {
  min: number;
  max: number;
  factor: number;
  center: number;
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
    }

    return skewed_proportion * (this.max - this.min) + this.min
  };

  constructor(min: number, max: number, factor: number, center: number) {
    this.min = min;
    this.max = max;
    this.factor = factor;
    this.center = center;
  }
}

export class ReversedRange implements FloatRange {
  range: FloatRange;
  getMin = () => this.range.getMax();
  getMax = () => this.range.getMin();

  clamp = (n: number) => this.range.clamp(n);
  normalize = (n: number) => 1 - this.range.normalize(n);
  unnormalize = (n: number) => this.range.unnormalize(1 - n);

  constructor(range: FloatRange) {
    this.range = range;
  }
}*/

const clamp = (n: number, min: number, max: number): number => {
  if (n < min) return min;
  if (n > max) return max;
  return n;
}