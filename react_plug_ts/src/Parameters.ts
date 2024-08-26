import {type Dispatch, type SetStateAction, useEffect, useState} from "react";
import {sendToPlugin} from "./util";
import type {FloatRange} from "./Ranges";
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
export type ParamOptions = {
  /** The unit label for this parameter, if any. */
  unit?: string,
  /**
   * This parameter’s polyphonic modulation ID. If this is set for a parameter in a
   * CLAP plugin, then polyphonic modulation will be enabled for that parameter.
   */
  polyModulationId?: number,
  /** Flags for controlling a parameter's behavior. */
  flags?: ParamFlags,
}

interface Parameter<T> {
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
}

export class FloatParam implements Parameter<number> {
  id: string;
  name: string;
  unit?: string | undefined;
  polyModulationId?: number | undefined;
  value: number;
  normalizedValue: number;
  defaultPlainValue: number;
  defaultNormalizedValue: number;
  stepCount?: number | undefined;
  flags: ParamFlags;
  previousStep: (from: number, finer?: boolean) => number;
  nextStep: (from: number, finer?: boolean) => number;
  previousNormalizedStep: (from: number, finer: boolean) => number;
  nextNormalizedStep: (from: number, finer: boolean) => number;
  previewNormalized: (plain: number) => number;
  previewPlain: (normalized: number) => number;

  setValue: (value: number) => void;
  setNormalizedValue: (value: number) => void;
  _setNormalizedValue: (value: number) => void;

  format: Formatter<number>;

  constructor(
    id: string,
    name: string,
    defaultValue: number,
    range: FloatRange,
    options: ParamOptions & {
      stepSize?: number,
      formatter?: Formatter<number>,
    }
  ) {
    this.id = id;
    this.name = name;
    this.unit = options.unit;
    this.polyModulationId = options.polyModulationId;
    this.flags = options.flags || {};
    this.format = options.formatter || ((n) => n.toString());

    this.previewNormalized = range.normalize;
    this.previewPlain = range.unnormalize;

    const normalizedDefaultValue = range.normalize(defaultValue);

    this.defaultPlainValue = defaultValue;
    this.defaultNormalizedValue = normalizedDefaultValue;

    const [value, setValue] = useState(defaultValue);
    const [normalizedValue, setNormalizedValue] = useState(normalizedDefaultValue);
    this._setNormalizedValue = (value) => {
      if (value == this.normalizedValue) return;
      setValue(this.previewPlain(value));
      setNormalizedValue(value);
    };

    this.value = value;
    this.normalizedValue = normalizedValue;

    this.setValue = (value: number) => {
      if (value == this.value) return;
      sendToPlugin({ParamChange: {id, value: this.previewNormalized(value)}});
      setValue(value);
      setNormalizedValue(this.previewNormalized(value));
    }

    this.setNormalizedValue = (value: number) => {
      if (value == this.normalizedValue) return;
      sendToPlugin({ParamChange: {id, value}});
      setValue(this.previewPlain(value));
      setNormalizedValue(value);
    }

    this.previousStep = (from, finer) => range.previousStep(from, options.stepSize, finer);
    this.nextStep = (from, finer) => range.nextStep(from, options.stepSize, finer);

    this.previousNormalizedStep = (from, finer) => this.previewNormalized(this.previousStep(this.previewPlain(from), finer));
    this.nextNormalizedStep = (from, finer) => this.previewNormalized(this.nextStep(this.previewPlain(from), finer));
  }
}