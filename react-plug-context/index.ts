import { createContext, useContext } from 'react';

export type FloatParam = {
  type: 'FloatParam';
  name: string;
  id: string;
  value: number;
  setValue: (newValue: number) => void;
  format: (value: number) => string;
  suffix: string;
};

export type IntParam = {
  type: 'IntParam';
  name: string;
  id: string;
  value: number;
  setValue: (newValue: number) => void;
  format: (value: number) => string;
  suffix: string;
};

export type BooleanParameter = {
  type: 'BooleanParam';
  name: string;
  id: string;
  value: boolean;
  setValue: (newValue: boolean) => void;
  format: (value: boolean) => string;
  suffix: string;
};

export type Parameter = FloatParam | IntParam | BooleanParameter;

export interface ContextType {
  parameters: Parameter[];
}

export const PluginContext = createContext<ContextType | undefined>(undefined);

export const usePluginContext = () => {
  const context = useContext(PluginContext);
  if (!context) {
    throw new Error('usePluginContext must be used within a provider');
  }
  return context;
};