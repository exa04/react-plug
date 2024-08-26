import {GUIMessage} from "./index";

export const MINUS_INFINITY_GAIN = 1e-5;
export const MINUS_INFINITY_DB = -100;
export const NOTES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

export const db_to_gain = (dbs: number) => {
  if (dbs > MINUS_INFINITY_DB)
    return Math.pow(10, (dbs * 0.05));
  else return 0;
}
export const gain_to_db = (gain: number) => Math.log10(Math.max(gain, MINUS_INFINITY_GAIN)) * 20;
export const db_to_gain_fast = (dbs: number) => Math.exp(dbs * Math.LN10 / 20);
export const db_to_gain_fast_branching = (dbs: number) => {
  if (dbs > MINUS_INFINITY_DB)
    db_to_gain_fast(dbs);
  else return 0;
}
export const gain_to_db_fast = (gain: number) => Math.log(Math.max(gain, MINUS_INFINITY_GAIN)) * Math.LOG10E * 20;
export const gain_to_db_fast_epsilon = (gain: number) => Math.log(Math.max(gain, MINUS_INFINITY_GAIN)) * Math.LOG10E * 20;
export const midi_note_to_freq = (note: number) => Math.pow(2, (note - 69) / 12) * 440;
export const freq_to_midi_note = (freq: number) => (Math.log2(freq / 440) * 12) + 69;

export function sendToPlugin<M>(message: GUIMessage<M>) {
  if ((window as any).ipc === undefined) {
    console.error("No IPC found!")
    return;
  }

  console.debug("Message (GUI -> Plugin)", message);

  (window as any).ipc.postMessage(JSON.stringify(message));
}

export const isParameterChange = (message: Object): message is { "ParameterChange": any } => {
  return typeof message === "object" && "ParameterChange" in message;
}