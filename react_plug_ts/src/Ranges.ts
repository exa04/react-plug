export interface FloatRange {
  min: number,
  max: number,
  clamp: (n: number) => number,
  normalize: (n: number) => number,
  unnormalize: (n: number) => number,
  snapToStep: (value: number, stepSize: number) => number,
  previousStep: (from: number, stepSize?: number, finer?: boolean) => number,
  nextStep: (from: number, stepSize?: number, finer?: boolean) => number,
}

export class LinearFloatRange implements FloatRange {
  min: number;
  max: number;

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
      result = this.snapToStep(naive_step, stepSize)
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
      result = this.snapToStep(naive_step, stepSize)
    } else {
      result = from + stepSize
    }

    return clamp(result, this.min, this.max);
  };

  snapToStep = (value: number, stepSize: number) => clamp(Math.round(value / stepSize) * stepSize, this.min, this.max);

  constructor(min: number, max: number) {
    this.min = min;
    this.max = max;
  }
}

export class SkewedFloatRange implements FloatRange {
  min: number;
  max: number;
  factor: number;

  clamp = (n: number) => clamp(n, this.min, this.max);
  normalize = (n: number) => Math.pow((this.clamp(n) - this.min) / (this.max - this.min), this.factor);
  unnormalize = (n: number) => Math.pow(n, 1 / this.factor) * (this.max - this.min) + this.min;

  previousStep = (from: number, stepSize?: number, finer?: boolean) => {
    const normalized_naive_step_size = finer ? 0.005 : 0.02;
    const naive_step = this.unnormalize(this.normalize(from) - normalized_naive_step_size);

    let result;
    if (stepSize === undefined) {
      result = naive_step
    } else if (Math.abs(naive_step - from) > stepSize) {
      result = this.snapToStep(naive_step, stepSize)
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
      result = this.snapToStep(naive_step, stepSize)
    } else {
      result = from + stepSize
    }

    return clamp(result, this.min, this.max);
  };

  snapToStep = (value: number, stepSize: number) => clamp(Math.round(value / stepSize) * stepSize, this.min, this.max);

  constructor(min: number, max: number, factor: number) {
    this.min = min;
    this.max = max;
    this.factor = factor;
  }
}

export class SymmetricalSkewedFloatRange implements FloatRange {
  min: number;
  max: number;
  factor: number;
  center: number;

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

  previousStep = (from: number, stepSize?: number, finer?: boolean) => {
    const normalized_naive_step_size = finer ? 0.005 : 0.02;
    const naive_step = this.unnormalize(this.normalize(from) - normalized_naive_step_size);

    let result;
    if (stepSize === undefined) {
      result = naive_step
    } else if (Math.abs(naive_step - from) > stepSize) {
      result = this.snapToStep(naive_step, stepSize)
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
      result = this.snapToStep(naive_step, stepSize)
    } else {
      result = from + stepSize
    }

    return clamp(result, this.min, this.max);
  };

  snapToStep = (value: number, stepSize: number) => clamp(Math.round(value / stepSize) * stepSize, this.min, this.max);

  constructor(min: number, max: number, factor: number, center: number) {
    this.min = min;
    this.max = max;
    this.factor = factor;
    this.center = center;
  }
}

export class ReversedFloatRange implements FloatRange {
  range: FloatRange;
  min;
  max;

  clamp;
  normalize = (n: number) => 1 - this.range.normalize(n);
  unnormalize = (n: number) => this.range.unnormalize(1 - n);

  previousStep;
  nextStep;

  snapToStep = (value: number, stepSize: number) => this.range.snapToStep(value, stepSize);

  constructor(range: FloatRange) {
    this.range = range;
    this.previousStep = range.nextStep;
    this.nextStep = range.previousStep;
    this.min = range.max;
    this.max = range.min;
    this.clamp = range.clamp;
  }
}

export interface IntRange {
  min: number;
  max: number;
  clamp: (n: number) => number,
  normalize: (n: number) => number,
  unnormalize: (n: number) => number,
  previousStep: (from: number) => number,
  nextStep: (from: number) => number,
  stepCount: number,
}

export class LinearIntRange implements IntRange {
  min;
  max;

  clamp = (n: number) => clamp(n, this.min, this.max);
  normalize = (n: number) => (this.clamp(n) - this.min) / (this.max - this.min);
  unnormalize = (n: number) => this.clamp(Math.round(n * (this.max - this.min)) + this.min);

  previousStep = (from: number) => clamp(from - 1, this.min, this.max);
  nextStep = (from: number) => clamp(from + 1, this.min, this.max);

  stepCount;

  constructor(min: number, max: number) {
    this.min = min;
    this.max = max;
    this.stepCount = this.max - this.min;
  }
}

export class ReversedIntRange implements IntRange {
  range;
  min;
  max;
  clamp;

  normalize = (n: number) => 1 - this.range.normalize(n);
  unnormalize = (n: number) => this.range.unnormalize(1 - n);

  previousStep;
  nextStep;
  stepCount;

  constructor(range: IntRange) {
    this.range = range;
    this.min = this.range.max;
    this.max = this.range.min;
    this.clamp = this.range.clamp;

    this.previousStep = this.range.nextStep;
    this.nextStep = this.range.previousStep;
    this.stepCount = this.range.stepCount;
  }
}

const clamp = (n: number, min: number, max: number): number => {
  if (n < min) return min;
  if (n > max) return max;
  return n;
}