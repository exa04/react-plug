import {type Dispatch, type SetStateAction, useEffect, useState} from "react";
import {sendToPlugin} from "./util";
import type {Range} from "./Ranges";
import type {Formatter} from "./Formatters";

interface Parameter<T> {
  type: string,
  id: string, // This ID will be used to receive and send param updates
  name: string,
  defaultValue: T,
  value: any,
  setValue: Dispatch<SetStateAction<T>> | ((value: T) => void),
}

export class FloatParam implements Parameter<number> {
  type = 'FloatParam';

  id: string;
  name: string;

  defaultValue: number;

  value: string;
  rawValue: number;

  _setDisplayedValue: Dispatch<SetStateAction<number>>;
  setValue: ((value: number) => void);

  formatter: Formatter<number>;

  range: Range;
  unit?: string;
  stepSize?: number;

  constructor(
    id: string,
    name: string,
    defaultValue: number,
    range: Range,
    options?: {
      unit?: string,
      stepSize?: number,
      formatter?: Formatter<number>,
    }
  ) {
    this.id = id;
    this.name = name;
    this.defaultValue = defaultValue;
    this.range = range;
    this.unit = options?.unit;
    this.stepSize = options?.stepSize;
    this.formatter = options?.formatter ?? (value => value.toFixed(2));

    [this.rawValue, this._setDisplayedValue] = useState(defaultValue);

    const [value, setValue] = useState(this.formatter(defaultValue));
    this.value = value;

    useEffect(() => {
      setValue(this.formatter(this.rawValue));
    }, [this.rawValue]);

    this.setValue = (value) => {
      if (value == this.rawValue) return;
      sendToPlugin({"ParameterChange": {[id]: value}})
      this._setDisplayedValue(value);
    }
  }
}

export class IntParam implements Parameter<number> {
  type = 'IntParam';

  id: string;
  name: string;

  defaultValue: number;

  value: string;
  rawValue: number;

  _setDisplayedValue: Dispatch<SetStateAction<number>>;
  setValue: ((value: number) => void);

  formatter: Formatter<number>;

  range: Range;
  unit?: string;

  constructor(
    id: string,
    name: string,
    defaultValue: number,
    range: Range,
    options?: {
      unit?: string,
      formatter?: Formatter<number>,
    }
  ) {
    this.id = id;
    this.name = name;

    this.defaultValue = defaultValue;

    const [rawValue, setRawValue] = useState(defaultValue);
    this.rawValue = rawValue;
    this._setDisplayedValue = setRawValue;

    this.unit = options?.unit;
    this.formatter = options?.formatter ?? (value => value.toFixed());

    const [value, setValue] = useState(this.formatter(defaultValue));
    this.value = value;

    this.range = range;

    useEffect(() => {
      setValue(this.formatter(this.rawValue));
    }, [this.rawValue]);

    this.setValue = (value) => {
      value = Math.floor(value);
      if (value == this.rawValue) return;
      sendToPlugin({"ParameterChange": {[id]: value}})
      setRawValue(value);
    }
  }
}

export class BoolParam implements Parameter<boolean> {
  type = 'BoolParam';
  id: string;
  name: string;
  defaultValue: boolean;
  rawValue: boolean;
  value: string;

  setValue: ((value: boolean) => void);
  _setDisplayedValue: Dispatch<SetStateAction<boolean>>;

  unit?: string;
  formatter: Formatter<boolean>;

  toggle = () => {
    this.setValue(!this.rawValue)
  };

  constructor(
    id: string,
    name: string,
    defaultValue: boolean,
    options?: {
      unit?: string,
      formatter?: Formatter<boolean>,
    }
  ) {
    this.id = id;
    this.name = name;

    this.defaultValue = defaultValue;

    const [rawValue, setRawValue] = useState(defaultValue);
    this.rawValue = rawValue;
    this._setDisplayedValue = setRawValue;

    this.unit = options?.unit;
    this.formatter = options?.formatter ?? (value => value ? "on" : "off");

    const [value, setValue] = useState(this.formatter(defaultValue));
    this.value = value;

    useEffect(() => {
      setValue(this.formatter(this.rawValue));
    }, [this.rawValue]);

    this.setValue = (value: boolean) => {
      sendToPlugin({"ParameterChange": {[id]: value}})
      setRawValue(value);
    }
  }
}

export class EnumParam implements Parameter<string> {
  type = 'EnumParam';
  id: string;
  name: string;

  variants: { [id: string]: string };

  defaultValue: string;
  rawValue: string;
  value: string;

  setValue: ((value: string) => void);
  _setDisplayedValue: Dispatch<SetStateAction<string>>;

  constructor(id: string, name: string, defaultValue: string, variants: { [id: string]: string }) {
    this.id = id;
    this.name = name;
    this.defaultValue = defaultValue;
    this.variants = variants;

    const [rawValue, setRawValue] = useState(defaultValue);
    this.rawValue = rawValue;
    this._setDisplayedValue = setRawValue;

    const [value, setValue] = useState(this.variants[defaultValue]);
    this.value = value;

    useEffect(() => {
      setValue(this.variants[this.rawValue]);
    }, [this.rawValue]);

    this.setValue = (value: string) => {
      sendToPlugin({"ParameterChange": {[id]: value}})
      setValue(value);
    }
  }
}
