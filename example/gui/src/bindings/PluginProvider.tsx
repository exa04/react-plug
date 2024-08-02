import {createContext, type Dispatch, FC, type SetStateAction, useContext, useEffect, useState} from 'react';

declare global {
  interface Window {
    ipc: { postMessage: (message: string) => void };
    onPluginMessage: (message: Object) => void;
  }
}

function sendToPlugin(msg: any) {
  if(window.ipc === undefined) {
    console.warn("No IPC found!")
    return;
  }
  window.ipc.postMessage(JSON.stringify(msg));
}

interface Parameter<T> {
  type: string,
  id: string, // This ID will be used to receive and send param updates
  name: string,
  defaultValue: T,
  value: T,
  setValue: Dispatch<SetStateAction<T>> | ((value: T) => void)
  _setDisplayedValue: Dispatch<SetStateAction<T>>; // would love to make this package-private, but it's not possible in JS :(
}

class FloatParam implements Parameter<number> {
  type = 'FloatParam';
  id: string;
  name: string;
  defaultValue: number;
  value: number;
  rawValue: number;
  setValue: ((value: number) => void);
  range: [number, number];
  _setDisplayedValue: Dispatch<SetStateAction<number>>;

  constructor(
    id: string,
    name: string,
    defaultValue: number,
    range: [number, number],
  ) {
    this.id = id;
    this.name = name;

    this.defaultValue = defaultValue;
    const [rawValue, setValue] = useState(defaultValue);
    this._setDisplayedValue = setValue;
    this.value = defaultValue;

    this.range = range;

    useEffect(() => {
      this.value = this.rawValue; // TODO: Formatting etc.
    }, [rawValue]);

    this.rawValue = rawValue;

    this.setValue = (value: number) => {
      sendToPlugin({ "ParameterChange": { [id]: value } })
      setValue(value);
    }
  }
}

class BoolParam implements Parameter<boolean> {
  type = 'BoolParam';
  id: string;
  name: string;
  defaultValue: boolean;
  value: boolean;
  setValue: Dispatch<SetStateAction<boolean>>;
  _setDisplayedValue: Dispatch<SetStateAction<boolean>>;

  constructor(
    id: string,
    name: string,
    defaultValue: boolean,
  ) {
    this.id = id;
    this.name = name;
    this.defaultValue = defaultValue;
    [this.value, this.setValue] = useState(defaultValue);
    this._setDisplayedValue = this.setValue;
  }
}

interface ContextType {
  parameters: Params
}

type Params = {
  gain: FloatParam,
  boolTest: BoolParam,
  // TODO: Codegen
};

const PluginContext = createContext<ContextType | undefined>(undefined);

const PluginProvider: FC<{ children: React.ReactNode }> = ({ children }) => {
  const parameters: Params  = {
    gain: new FloatParam("Gain", "Gain", 1, [0.5, 2]),
    boolTest: new BoolParam("BoolTest", "Bool Test", false)
    // TODO: Codegen
  };

  const isParameterChange = (message: Object): message is { "ParameterChange": any } => {
    return typeof message === "object" && "ParameterChange" in message;
  }

  useEffect(() => {
    sendToPlugin("Init");

    window.onPluginMessage = (message: Object) => {
      console.log(message);
      if(isParameterChange(message)) {
        const [id, value] = Object.entries(message.ParameterChange)[0];

        const param = Object.values(parameters)
          .find((p) => p.id == id);

        if(param === undefined)
          throw new Error('usePluginContext must be used within a provider');

        if(param.type == "FloatParam")
          (param as FloatParam)._setDisplayedValue(value as unknown as number);
        else if(param.type == "BoolParam")
          (param as BoolParam)._setDisplayedValue(value as unknown as boolean);
      }
    };
  }, []);

  return (
    <PluginContext.Provider value={{ parameters }}>
      {children}
    </PluginContext.Provider>
  );
};

export const usePluginContext = () => {
  const context = useContext(PluginContext);
  if (!context) {
    throw new Error('usePluginContext must be used within a provider');
  }
  return context;
};

export default PluginProvider;