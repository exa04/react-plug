import {type Dispatch, type SetStateAction, useEffect, useState} from "react";
import {sendToPlugin} from "./util";
import type {FloatRange, IntRange} from "./Ranges";
import type {Formatter} from "./Formatters";

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
  value_to_string?: Formatter<T>
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
  value_to_string: Formatter<T>;
}

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

  value_to_string: Formatter<number>;

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
    range: FloatRange,
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

  value_to_string: Formatter<number>;

  constructor({id, name, defaultValue, range, value_to_string, unit, polyModulationId, flags}: ParamOptions<number> & {
    range: IntRange,
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
  previousStep = (from: boolean) => false;
  nextStep = (from: boolean) => true;
  previousNormalizedStep = (from: number) => 0;
  nextNormalizedStep = (from: number) => 1;
  previewNormalized = (plain: boolean) => plain ? 1 : 0;
  previewPlain = (normalized: number) => normalized > 0.5;

  setValue: (value: boolean) => void;
  resetValue: () => void;
  setNormalizedValue: (value: number) => void;
  _setNormalizedValue: (value: number) => void;

  value_to_string: Formatter<boolean>;

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

  value_to_string: Formatter<string>;

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