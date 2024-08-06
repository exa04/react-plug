import {createContext, FC, ReactNode, useContext, useEffect, useRef} from 'react';

import { EventEmitter } from 'events';

import * as params from 'react-plug/Parameters';
import * as ranges from 'react-plug/Ranges';
import * as formatters from 'react-plug/Formatters';
import { sendToPlugin, isParameterChange } from 'react-plug/util';

import {GuiMessage} from "./GuiMessage";
import {PluginMessage} from "./PluginMessage";

interface ContextType {
  parameters: Params;
  sendToPlugin: (message: GuiMessage) => void;
  addMessageListener: (action: (message: PluginMessage) => void) => void;
  removeMessageListener: (action: (message: PluginMessage) => void) => void;
}

const PluginContext = createContext<ContextType | undefined>(undefined);

type Params = {
  gain: params.FloatParam,
  boolTest: params.BoolParam,
  intTest: params.IntParam
};

const PluginProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const eventEmitter = useRef(new EventEmitter());

  const addMessageListener = (action: (message: PluginMessage) => void) => eventEmitter.current.on('pluginMessage', action);
  const removeMessageListener = (action: (message: PluginMessage) => void) => eventEmitter.current.off('pluginMessage', action);

  const parameters: Params = {
    gain: new params.FloatParam("Gain", "Gain", 1, new ranges.LinearRange(0.001, 1), { unit: " dB", formatter: formatters.v2s_f32_gain_to_db(2), }), boolTest: new params.BoolParam("BoolTest", "Bool Test", false, { }), intTest: new params.IntParam("IntTest", "Int Test", 0, new ranges.LinearRange(0, 10), { }), 
  };

  useEffect(() => {
    sendToPlugin('Init');

    // TODO: This kinda sucks
    (window as any).onPluginMessage = (message: Object) => {
      console.log('Message (Plugin -> GUI)', message);
      if(isParameterChange(message)) {
        const [id, value] = Object.entries(message.ParameterChange)[0];

        const param = Object.values(parameters)
          .find((p) => p.id == id);

        if(param === undefined)
          throw new Error('usePluginContext must be used within a provider');

        if(param.type == 'FloatParam')
          (param as params.FloatParam)._setDisplayedValue(value as unknown as number);
        else if(param.type == 'IntParam')
          (param as params.IntParam)._setDisplayedValue(value as unknown as number);
        else if(param.type == 'BoolParam')
          (param as params.BoolParam)._setDisplayedValue(value as unknown as boolean);
      } else {
        eventEmitter.current.emit('pluginMessage', message as PluginMessage);
      }
    };
  }, []);

  return (
    <PluginContext.Provider value={{
      parameters,
      sendToPlugin,
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
