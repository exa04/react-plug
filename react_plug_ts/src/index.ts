export * as Formatters from './Formatters';
export * as Parameters from './Parameters';
export * as Ranges from './Ranges';
export * as util from './util';

export type ParamChange = { id: string, value: number }

export type PluginMessage<M> =
  { "ParamChange": ParamChange } |
  { "Message": M };

export type GUIMessage<M> =
  { "ParamChange": ParamChange } |
  "Init" |
  { "Message": M };