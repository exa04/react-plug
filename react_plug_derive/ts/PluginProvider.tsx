import {createContext, FC, ReactNode, useContext, useEffect, useRef} from 'react';
import {EventEmitter} from 'events';

import {type Params, createParameters} from './Params';
import {type GuiMessage} from "./GuiMessage.ts";
import {type PluginMessage} from "./PluginMessage.ts";
import * as ReactPlug from "./react-plug.ts";

interface ContextType {
  parameters: Params;
  sendToPlugin: (message: GuiMessage) => void;
  addMessageListener: (action: (message: PluginMessage) => void) => void;
  removeMessageListener: (action: (message: PluginMessage) => void) => void;
}

const PluginContext = createContext<ContextType | undefined>(undefined);

const PluginProvider: FC<{ children: ReactNode }> = ({children}) => {
  const eventEmitter = useRef(new EventEmitter());

  const addMessageListener = (action: (message: PluginMessage) => void) => eventEmitter.current.on('pluginMessage', action);
  const removeMessageListener = (action: (message: PluginMessage) => void) => eventEmitter.current.off('pluginMessage', action);
  const parameters = createParameters();

  useEffect(() => {
    ReactPlug.sendToPlugin('Init');

    (window as unknown as ReactPlug.Window).onPluginMessage = (message: ReactPlug.PluginMessage<unknown>) => {
      if ("ParamChange" in message) {
        const paramChange = (message.ParamChange as ReactPlug.ParamChange)
        console.log("Parameter change (Plugin -> GUI)", paramChange);

        Object.values(parameters).find(param => param.id == paramChange.id)?._setNormalizedValue(paramChange.value);
      } else if ("Message" in message) {
        eventEmitter.current.emit('pluginMessage', message.Message)
      }
    };
  }, []);

  return (
    <PluginContext.Provider value={{
      parameters,
      sendToPlugin: (message: GuiMessage) => {
        console.log("Message", message)
        ReactPlug.sendToPlugin({"Message": message})
      },
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
