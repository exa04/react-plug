import {freq_to_midi_note, MINUS_INFINITY_GAIN, NOTES} from "./util";

export type Formatter<T> = (value: T) => string;

export const v2s_f32_rounded = (digits: number): Formatter<number> => (n: number) => n.toFixed(digits);
export const v2s_f32_percentage = (digits?: number): Formatter<number> => (n: number) => (n * 100).toFixed(digits);
export const v2s_compression_ratio = (digits?: number): Formatter<number> => (n: number) => {
  if (n >= 1.0) return n.toFixed(digits)
  else return (1 / n).toFixed(digits)
}
export const v2s_f32_gain_to_db = (digits?: number): Formatter<number> => (n: number) => {
  if(n < MINUS_INFINITY_GAIN) return "-inf";
  return (20 * Math.log10(n)).toFixed(digits)
};
export const v2s_f32_panning = (digits?: number): Formatter<number> => (n: number) => {
  if(n === 0) return "C";
  if(n < 0) return `L${(n * -100).toFixed(digits)}`;
  if(n > 0) return `R${(n * 100).toFixed(digits)}`;
  return "NaN";
}
export const v2s_f32_hz_then_khz = (digits?: number): Formatter<number> => (n: number) => {
  if(n < 1000) return `${n.toFixed(digits)} Hz`;
  return `${(n / 1000).toFixed(digits)} kHz`;
}
export const v2s_f32_hz_then_khz_with_note_name = (digits: number, include_cents: boolean): Formatter<number> => (n: number) => {
  if (Math.abs(n) < 1) return `${n.toFixed(digits)}Hz`;

  const fractional_note = freq_to_midi_note(n);
  const note = Math.round(fractional_note);
  const cents = Math.round((fractional_note - note) * 100);
  const noteName = NOTES[note % 12];
  const octave = Math.floor(note / 12) - 1;

  let noteString: string;
  if (cents == 0 || !include_cents) {
    noteString = `${noteName}${octave}`;
  } else {
    noteString = `${noteName}${octave}, ${cents} ct.`;
  }

  if (n < 1000) return `${(n).toFixed(digits)} kHz (${noteString})`;
  else return `${(n / 1000).toFixed(digits)} kHz, ${noteString}`;
}
export const v2s_i32_power_of_two = (): Formatter<number> => (n: number) => (1 << n).toString();
export const v2s_i32_note_formatter = (): Formatter<number> => (n: number) => {
  const note_name = NOTES[n % 12];
  const octave = (n / 12) - 1;
  return `${note_name}${octave}`;
}
export const v2s_bool_bypass = (): Formatter<boolean> => (n: boolean) => n ? "Bypassed" : "Not Bypassed";