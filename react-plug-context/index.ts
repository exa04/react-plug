import {createContext, type Dispatch, type SetStateAction, useContext, useState} from 'react';

export interface Parameter {
  type: string,
  name: string,
  rawValue: any,
  setValue: Dispatch<SetStateAction<any>>
}

export class FloatParam implements Parameter {
  type = 'FloatParam';
  name: string;
  rawValue: number;
  setValue: Dispatch<SetStateAction<number>>;
  range: [number, number];

  constructor(
    name: string,
    defaultValue: number,
    range: [number, number],
  ) {
    this.name = name;
    [this.rawValue, this.setValue] = useState(defaultValue);
    this.range = range;
  }
}

export interface ContextType {
  parameters: {[id: string]: Parameter};
}

export const PluginContext = createContext<ContextType | undefined>(undefined);

export const usePluginContext = () => {
  const context = useContext(PluginContext);
  if (!context) {
    throw new Error('usePluginContext must be used within a provider');
  }
  return context;
};

export const getParameter = (name: string) => {
  const {parameters} = usePluginContext();
  if (!parameters[name]) {
    throw new Error(`Parameter ${name} not found`);
  }
  return parameters[name];
}