/* eslint-disable @typescript-eslint/no-namespace */
/* eslint-disable react-hooks/rules-of-hooks */
import {useState} from "react";

export type ParamChange = { id: string, value: number }

export type PluginMessage<M> =
  { "ParamChange": ParamChange } |
  { "Message": M };

export type GUIMessage<M> =
  { "ParamChange": ParamChange } |
  "Init" |
  { "Message": M };

export type ValueToString<T> = (value: T) => string;

export interface Window {
  ipc: { postMessage: (message: string) => void };
  onPluginMessage: (message: PluginMessage<unknown>) => void;
}

export function sendToPlugin<M>(message: GUIMessage<M>) {
  if ((window as unknown as Window).ipc === undefined) {
    console.error("No IPC found!")
    return;
  }

  console.debug("Message (GUI -> Plugin)", message);

  (window as unknown as Window).ipc.postMessage(JSON.stringify(message));
}

export interface Parameter<T> {
  /** The unique identifier for this parameter. */
  id: string;
  /** The human-readable name for this parameter. */
  name: string;
  /** The unit label for this parameter, if any. */
  unit?: string;
  /**
   * This parameter’s polyphonic modulation ID. If this is set for a parameter in a
   * CLAP plugin, then polyphonic modulation will be enabled for that parameter.
   */
  polyModulationId?: number;

  // = PLAIN VALUES ============================================================= //

  /** The current value for this parameter. The same as `modulatedPlainValue`. */
  value: T;
  /** Set the current value for this parameter. */
  setValue: (value: T) => void;
  /** Reset the current value for this parameter to its default value. */
  resetValue: () => void;
  /** The unnormalized default value for this parameter. */
  defaultPlainValue: T;

  // = NORMALIZED VALUES ======================================================== //

  /** The current normalized value for this parameter. */
  normalizedValue: number;
  /** Set the current normalized value for this parameter. */
  setNormalizedValue: (value: number) => void;
  /**
   * Set the current normalized value for this parameter. This is used internally
   * by the React-Plug framework and should not be called directly.
   */
  _setNormalizedValue: (value: number) => void;
  /** Get the normalized [0, 1] default value for this parameter. */
  defaultNormalizedValue: number;

  // = STEPPING ================================================================= //

  /** The number of steps for this parameter, if it is discrete. Used for the host’s generic UI. */
  stepCount?: number;
  /**
   * The previous step from a specific value for this parameter. This can be the
   * same as from if the value is at the start of its range. This is mainly used for
   * scroll wheel interaction in plugin GUIs. When the parameter is not discrete
   * then a step should cover one hundredth of the normalized range instead.
   *
   * If finer is true, then the step size should be decreased if the parameter is
   * continuous.
   */
  previousStep: (from: T, finer?: boolean) => T;
  /**
   * Returns the next step from a specific value for this parameter. This can be the
   * same as from if the value is at the end of its range. This is mainly used for
   * scroll wheel interaction in plugin GUIs. When the parameter is not discrete
   * then a step should cover one hundredth of the normalized range instead.
   *
   * If finer is true, then the step size should be decreased if the parameter is
   * continuous.
   */
  nextStep: (from: T, finer?: boolean) => T;
  /**
   * The same as `previous_step()`, but for normalized values. This is mostly useful
   * for GUI widgets.
   */
  previousNormalizedStep: (from: number, finer: boolean) => number;
  /**
   * The same as next_step(), but for normalized values. This is mostly useful for
   * GUI widgets.
   */
  nextNormalizedStep: (from: number, finer: boolean) => number;
  /** Get the normalized value for a plain, unnormalized value, as a float. */
  previewNormalized: (plain: T) => number;
  /**
   * Get the plain, unnormalized value for a normalized value, as a float. Used as
   * part of the wrappers. This does snap to step sizes for continuous parameters
   * (i.e. FloatParam).
   */
  previewPlain: (normalized: number) => T;
  /**
   * Flags to control the parameter’s behavior.
   */
  flags: ParamFlags;
  value_to_string: ValueToString<T>;
}

/**
 * Flags for controlling a parameter’s behavior.
 */
export type ParamFlags = {
  /**
   * When applied to a [`BoolParam`], this will cause the parameter to be linked
   * to the host's bypass control. Only a single parameter can be marked as a
   * bypass parameter. If you don't have a bypass parameter, then NIH-plug will
   * add one for you. You will need to implement this yourself if your plugin
   * introduces latency.
   */
  bypass?: boolean,
  /**
   * If true, the parameter cannot be changed from an automation lane. The
   * parameter can however still be manually changed by the user from either the
   * plugin's own GUI or from the host's generic UI.
   */
  nonAutomatable?: boolean,
  /**
   * Don’t show this parameter when generating a generic UI for the plugin using
   * one of NIH-plug’s generic UI widgets.
   */
  hideInGenericUi?: boolean,
}

/**
 * Options for creating a new parameter.
 */
export type ParamOptions<T> = {
  id: string,
  name: string,
  defaultValue: T,
  value_to_string?: ValueToString<T>
  /** The unit label for this parameter. */
  unit?: string,
  /**
   * This parameter’s polyphonic modulation ID. If this is set for a parameter in a
   * CLAP plugin, then polyphonic modulation will be enabled for that parameter.
   */
  polyModulationId?: number,
  /** Flags for controlling a parameter's behavior. */
  flags?: ParamFlags,
}

export module util {
  export const MINUS_INFINITY_GAIN = 1e-5;
  export const MINUS_INFINITY_DB = -100;
  export const NOTES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

  export const round = (n: number, digits?: number): string => {
    if (digits != undefined) {
      const roundingMultiplier = Math.pow(10, digits);
      return (Math.round(roundingMultiplier * n) / roundingMultiplier).toFixed(digits)
    } else {
      return n.toString();
    }
  }
  export const db_to_gain = (dbs: number) => {
    if (dbs > MINUS_INFINITY_DB)
      return Math.pow(10, (dbs * 0.05));
    else return 0;
  }
  export const gain_to_db = (gain: number) => Math.log10(Math.max(gain, MINUS_INFINITY_GAIN)) * 20;
  export const db_to_gain_fast = (dbs: number) => Math.exp(dbs * Math.LN10 / 20);
  export const db_to_gain_fast_branching = (dbs: number) => {
    if (dbs > MINUS_INFINITY_DB)
      db_to_gain_fast(dbs);
    else return 0;
  }
  export const gain_to_db_fast = (gain: number) => Math.log(Math.max(gain, MINUS_INFINITY_GAIN)) * Math.LOG10E * 20;
  export const gain_to_db_fast_epsilon = (gain: number) => Math.log(Math.max(gain, MINUS_INFINITY_GAIN)) * Math.LOG10E * 20;
  export const midi_note_to_freq = (note: number) => Math.pow(2, (note - 69) / 12) * 440;
  export const freq_to_midi_note = (freq: number) => (Math.log2(freq / 440) * 12) + 69;
}

export module formatters {
  export const v2s_f32_rounded = (digits: number): ValueToString<number> => (n: number) => util.round(n, digits);
  export const v2s_f32_percentage = (digits?: number): ValueToString<number> => (n: number) => util.round(n * 100, digits);
  export const v2s_compression_ratio = (digits?: number): ValueToString<number> => (n: number) => {
    if (n >= 1.0) return util.round(n, digits);
    else return util.round(1 / n, digits)
  }
  export const v2s_f32_gain_to_db = (digits?: number): ValueToString<number> => (n: number) => {
    if (n < util.MINUS_INFINITY_GAIN) return "-inf";

    return util.round(20 * Math.log10(n), digits);
  };
  export const v2s_f32_panning = (digits?: number): ValueToString<number> => (n: number) => {
    if (n === 0) return "C";
    if (n < 0) return `L${util.round(n * -100, digits)}`;
    if (n > 0) return `R${util.round(n * 100, digits)}`;
    return "NaN";
  }
  export const v2s_f32_hz_then_khz = (digits?: number): ValueToString<number> => (n: number) => {
    if (n < 1000) return `${util.round(n, digits)} Hz`;
    return `${util.round(n, digits)} kHz`;
  }
  export const v2s_f32_hz_then_khz_with_note_name = (digits: number, include_cents: boolean): ValueToString<number> => (n: number) => {
    if (Math.abs(n) < 1) return `${util.round(n, digits)}Hz`;

    const fractional_note = util.freq_to_midi_note(n);
    const note = Math.round(fractional_note);
    const cents = Math.round((fractional_note - note) * 100);
    const noteName = util.NOTES[note % 12];
    const octave = Math.floor(note / 12) - 1;

    let noteString: string;
    if (cents == 0 || !include_cents) {
      noteString = `${noteName}${octave}`;
    } else {
      noteString = `${noteName}${octave}, ${cents} ct.`;
    }

    if (n < 1000) return `$round(n, digits)} kHz (${noteString})`;
    else return `${util.round(n / 1000, digits)} kHz, ${noteString}`;
  }
  export const v2s_i32_power_of_two = (): ValueToString<number> => (n: number) => (1 << n).toString();
  export const v2s_i32_note_formatter = (): ValueToString<number> => (n: number) => {
    const note_name = util.NOTES[n % 12];
    const octave = (n / 12) - 1;
    return `${note_name}${octave}`;
  }
  export const v2s_bool_bypass = (): ValueToString<boolean> => (n: boolean) => n ? "Bypassed" : "Not Bypassed";
}

export module ranges {
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

    constructor({min, max}: { min: number, max: number }) {
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

    constructor({min, max, factor}: { min: number, max: number, factor: number }) {
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

    constructor({min, max, factor, center}: { min: number, max: number, factor: number, center: number }) {
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

    constructor({min, max}: { min: number, max: number }) {
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
}

export module parameters {
  export class FloatParam implements Parameter<number> {
    id: string;
    name: string;
    unit?: string;
    polyModulationId?: number;
    value: number;
    normalizedValue: number;
    defaultPlainValue: number;
    defaultNormalizedValue: number;
    stepCount?: number;
    flags: ParamFlags;
    previousStep: (from: number, finer?: boolean) => number;
    nextStep: (from: number, finer?: boolean) => number;
    previousNormalizedStep: (from: number, finer: boolean) => number;
    nextNormalizedStep: (from: number, finer: boolean) => number;
    previewNormalized: (plain: number) => number;
    previewPlain: (normalized: number) => number;

    setValue: (value: number) => void;
    resetValue: () => void;
    setNormalizedValue: (value: number) => void;
    _setNormalizedValue: (value: number) => void;

    value_to_string: ValueToString<number>;

    constructor({
                  id,
                  name,
                  defaultValue,
                  range,
                  value_to_string,
                  stepSize,
                  unit,
                  polyModulationId,
                  flags
                }: ParamOptions<number> & {
      range: ranges.FloatRange,
      stepSize?: number,
    }) {
      this.id = id;
      this.name = name;
      this.unit = unit;
      this.polyModulationId = polyModulationId;
      this.flags = flags || {};
      this.value_to_string = value_to_string || ((n) => n.toFixed(2));

      this.previewNormalized = range.normalize;
      this.previewPlain = range.unnormalize;

      const defaultNormalizedValue = range.normalize(defaultValue);

      this.defaultPlainValue = defaultValue;
      this.defaultNormalizedValue = defaultNormalizedValue;

      const [value, setValue] = useState(defaultValue);
      const [normalizedValue, setNormalizedValue] = useState(defaultNormalizedValue);

      this.value = value;
      this.normalizedValue = normalizedValue;

      this._setNormalizedValue = (value) => {
        if (this.normalizedValue == value) return;

        setValue(this.previewPlain(value));
        setNormalizedValue(value);
      };

      this.setValue = (value: number) => {
        setValue(value);
        setNormalizedValue(this.previewNormalized(value));
        sendToPlugin({ParamChange: {id, value: this.previewNormalized(value)}});
      }

      this.setNormalizedValue = (value: number) => {
        setValue(this.previewPlain(value));
        setNormalizedValue(value);
        sendToPlugin({ParamChange: {id, value}});
      }

      this.resetValue = () => {
        this.setNormalizedValue(defaultNormalizedValue);
      }

      this.previousStep = (from, finer) => range.previousStep(from, stepSize, finer);
      this.nextStep = (from, finer) => range.nextStep(from, stepSize, finer);

      this.previousNormalizedStep = (from, finer) => this.previewNormalized(this.previousStep(this.previewPlain(from), finer));
      this.nextNormalizedStep = (from, finer) => this.previewNormalized(this.nextStep(this.previewPlain(from), finer));
    }
  }

  export class IntParam implements Parameter<number> {
    id: string;
    name: string;
    unit?: string;
    polyModulationId?: number;
    value: number;
    normalizedValue: number;
    defaultPlainValue: number;
    defaultNormalizedValue: number;
    stepCount?: number;
    flags: ParamFlags;
    previousStep: (from: number) => number;
    nextStep: (from: number) => number;
    previousNormalizedStep: (from: number) => number;
    nextNormalizedStep: (from: number) => number;
    previewNormalized: (plain: number) => number;
    previewPlain: (normalized: number) => number;

    setValue: (value: number) => void;
    resetValue: () => void;
    setNormalizedValue: (value: number) => void;
    _setNormalizedValue: (value: number) => void;

    value_to_string: ValueToString<number>;

    constructor({
                  id,
                  name,
                  defaultValue,
                  range,
                  value_to_string,
                  unit,
                  polyModulationId,
                  flags
                }: ParamOptions<number> & {
      range: ranges.IntRange,
    }) {
      this.id = id;
      this.name = name;
      this.unit = unit;
      this.polyModulationId = polyModulationId;
      this.flags = flags || {};
      this.value_to_string = value_to_string || ((n) => n.toFixed());

      this.previewNormalized = range.normalize;
      this.previewPlain = range.unnormalize;

      const defaultNormalizedValue = range.normalize(defaultValue);

      this.defaultPlainValue = defaultValue;
      this.defaultNormalizedValue = defaultNormalizedValue;

      const [value, setValue] = useState(defaultValue);
      const [normalizedValue, setNormalizedValue] = useState(defaultNormalizedValue);

      this.value = value;
      this.normalizedValue = normalizedValue;

      this._setNormalizedValue = (value) => {
        setValue(this.previewPlain(value));
        setNormalizedValue(value);
      };

      this.setValue = (value: number) => {
        setValue(value);
        setNormalizedValue(this.previewNormalized(value));
        sendToPlugin({ParamChange: {id, value: this.previewNormalized(value)}});
      }

      this.setNormalizedValue = (value: number) => {
        setValue(this.previewPlain(value));
        setNormalizedValue(value);
        sendToPlugin({ParamChange: {id, value}});
      }

      this.resetValue = () => {
        this.setNormalizedValue(defaultNormalizedValue);
      }

      this.previousStep = range.previousStep;
      this.nextStep = range.nextStep;
      this.stepCount = range.stepCount;

      this.previousNormalizedStep = (from) => this.previewNormalized(this.previousStep(this.previewPlain(from)));
      this.nextNormalizedStep = (from) => this.previewNormalized(this.nextStep(this.previewPlain(from)));
    }
  }

  export class BoolParam implements Parameter<boolean> {
    id: string;
    name: string;
    polyModulationId?: number;
    value: boolean;
    unit?: "";
    normalizedValue: number;
    defaultPlainValue: boolean;
    defaultNormalizedValue: number;
    stepCount?: 1;
    flags: ParamFlags;
    previousStep = () => false;
    nextStep = () => true;
    previousNormalizedStep = () => 0;
    nextNormalizedStep = () => 1;
    previewNormalized = (plain: boolean) => plain ? 1 : 0;
    previewPlain = (normalized: number) => normalized > 0.5;

    setValue: (value: boolean) => void;
    resetValue: () => void;
    setNormalizedValue: (value: number) => void;
    _setNormalizedValue: (value: number) => void;

    value_to_string: ValueToString<boolean>;

    constructor({id, name, defaultValue, polyModulationId, flags, value_to_string}: ParamOptions<boolean>) {
      this.id = id;
      this.name = name;
      this.polyModulationId = polyModulationId;
      this.flags = flags || {};
      this.value_to_string = value_to_string || ((n) => n ? "On" : "Off");

      const defaultNormalizedValue = defaultValue ? 1 : 0;

      this.defaultPlainValue = defaultValue;
      this.defaultNormalizedValue = defaultNormalizedValue;

      const [value, setValue] = useState(defaultValue);
      const [normalizedValue, setNormalizedValue] = useState(defaultNormalizedValue);

      this.value = value;
      this.normalizedValue = normalizedValue;

      this._setNormalizedValue = (value) => {
        setValue(this.previewPlain(value));
        setNormalizedValue(value);
      };

      this.setValue = (value: boolean) => {
        sendToPlugin({ParamChange: {id, value: this.previewNormalized(value)}});
        setValue(value);
        setNormalizedValue(this.previewNormalized(value));
      }

      this.setNormalizedValue = (value: number) => {
        sendToPlugin({ParamChange: {id, value}});
        setValue(this.previewPlain(value));
        setNormalizedValue(value);
      }

      this.resetValue = () => {
        this.setNormalizedValue(defaultNormalizedValue);
      }
    }
  }

  export class EnumParam implements Parameter<string> {
    id: string;
    name: string;
    polyModulationId?: number;
    value: string;
    variants: { [key: string]: string };
    unit?: "";
    normalizedValue: number;
    defaultPlainValue: string;
    defaultNormalizedValue: number;
    stepCount: number;
    flags: ParamFlags;
    previousStep: (from: string) => string;
    nextStep: (from: string) => string;
    previousNormalizedStep: (from: number) => number;
    nextNormalizedStep: (from: number) => number;
    previewNormalized: (plain: string) => number;
    previewPlain: (normalized: number) => string;

    setValue: (value: string) => void;
    resetValue: () => void;
    setNormalizedValue: (value: number) => void;
    _setNormalizedValue: (value: number) => void;

    value_to_string: ValueToString<string>;

    constructor({id, name, defaultValue, variants, polyModulationId, flags}: ParamOptions<string> & {
      variants: { [key: string]: string },
    }) {
      this.id = id;
      this.name = name;
      this.variants = variants;
      this.polyModulationId = polyModulationId;
      this.flags = flags || {};
      this.value_to_string = (value: string) => this.variants[value];

      this.stepCount = Object.keys(variants).length;

      this.previewNormalized = (plain: string) => Object.keys(this.variants).findIndex(id => id == plain) / (this.stepCount - 1);
      this.previewPlain = (normalized: number) => Object.keys(this.variants)[Math.round(normalized * (this.stepCount - 1))];

      const defaultNormalizedValue = this.previewNormalized(defaultValue);
      this.defaultNormalizedValue = defaultNormalizedValue;
      this.defaultPlainValue = defaultValue;

      const [value, setValue] = useState(defaultValue);
      const [normalizedValue, setNormalizedValue] = useState(defaultNormalizedValue);

      this.value = value;
      this.normalizedValue = normalizedValue;

      this._setNormalizedValue = (value) => {
        setValue(this.previewPlain(value));
        setNormalizedValue(value);
      }

      this.setValue = (value: string) => {
        setValue(value);
        setNormalizedValue(this.previewNormalized(value));
        sendToPlugin({ParamChange: {id, value: this.previewNormalized(value)}});
      }

      this.setNormalizedValue = (value: number) => {
        setValue(this.previewPlain(value));
        setNormalizedValue(value);
        sendToPlugin({ParamChange: {id, value}});
      }

      this.resetValue = () => {
        this.setNormalizedValue(defaultNormalizedValue);
      }

      this.previousStep = (from) => {
        const keys = Object.keys(this.variants);
        const index = keys.findIndex(id => id == from);
        if (index < 0) return from;
        return keys[index - 1];
      }

      this.nextStep = (from) => {
        const keys = Object.keys(this.variants);
        const index = keys.findIndex(id => id == from);
        if (index >= keys.length) return from;
        return keys[index + 1 % keys.length];
      }

      this.previousNormalizedStep = (from) => this.previewNormalized(this.previousStep(this.previewPlain(from)));

      this.nextNormalizedStep = (from) => this.previewNormalized(this.nextStep(this.previewPlain(from)));
    }
  }
}
