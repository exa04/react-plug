import {createContext, FC, ReactNode, useContext, useEffect, useRef} from 'react';

import {EventEmitter} from 'events';

import * as ReactPlug from '@exa04/react-plug';

interface ContextType {
  parameters: Params;
  sendToPlugin: (message: any) => void;
  addMessageListener: (action: (message: any) => void) => void;
  removeMessageListener: (action: (message: any) => void) => void;
}

const PluginContext = createContext<ContextType | undefined>(undefined);

type Params = {
  gain: ReactPlug.Parameters.FloatParam,
  reversed: ReactPlug.Parameters.FloatParam,
  intTest: ReactPlug.Parameters.IntParam,
  boolTest: ReactPlug.Parameters.BoolParam,
  enumTest: ReactPlug.Parameters.EnumParam,
};

const PluginProvider: FC<{ children: ReactNode }> = ({children}) => {
  const eventEmitter = useRef(new EventEmitter());

  const addMessageListener = (action: (message: any) => void) => eventEmitter.current.on('pluginMessage', action);
  const removeMessageListener = (action: (message: any) => void) => eventEmitter.current.off('pluginMessage', action);

  const parameters: Params = {
    gain: new ReactPlug.Parameters.FloatParam(
      "gain",
      "Gain",
      1,
      new ReactPlug.Ranges.LinearFloatRange(ReactPlug.util.db_to_gain(-60), ReactPlug.util.db_to_gain(6.0)),
      {
        unit: " dB",
        formatter: ReactPlug.Formatters.v2s_f32_gain_to_db(2),
      }
    ),
    reversed: new ReactPlug.Parameters.FloatParam(
      "reversed",
      "Reversed",
      0,
      new ReactPlug.Ranges.ReversedFloatRange(new ReactPlug.Ranges.LinearFloatRange(0, 1)),
      {}
    ),
    boolTest: new ReactPlug.Parameters.BoolParam(
      "bool_test",
      "Bool Test",
      false,
      {}
    ),
    intTest: new ReactPlug.Parameters.IntParam(
      "int_test",
      "Int Test",
      0,
      new ReactPlug.Ranges.LinearIntRange(0, 10),
      {}
    ),
    enumTest: new ReactPlug.Parameters.EnumParam(
      "enum_test",
      "Enum Test",
      "A",
      {"A": "A", "B": "Option B", "C": "Option C"},
      {}
    ),
  };

  useEffect(() => {
    ReactPlug.util.sendToPlugin('Init');

    (window as any).onPluginMessage = (message: ReactPlug.PluginMessage<any>) => {
      if ("ParamChange" in message) {
        const paramChange = (message.ParamChange as ReactPlug.ParamChange)
        console.log("Parameter change (Plugin -> GUI)", paramChange);

        Object.values(parameters).find(param => param.id == paramChange.id)?._setNormalizedValue(paramChange.value);
      } else {
        console.log('Message (Plugin -> GUI)', message);
      }
    };
  }, []);

  return (
    <PluginContext.Provider value={{
      parameters,
      sendToPlugin: ReactPlug.util.sendToPlugin,
      addMessageListener,
      removeMessageListener
    }}>
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
