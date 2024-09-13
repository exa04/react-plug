import * as ReactPlug from "@exa04/react-plug";

export type Params = {
    gain: ReactPlug.Parameters.FloatParam, 
    reversed: ReactPlug.Parameters.FloatParam, 
    bool_test: ReactPlug.Parameters.BoolParam, 
    int_test: ReactPlug.Parameters.IntParam, 
    wave_shape: ReactPlug.Parameters.EnumParam
};

export const createParameters: () => Params = () => ({
    gain: new ReactPlug.Parameters.FloatParam({ name: "Gain", defaultValue: 1, range: new ReactPlug.Ranges.SkewedFloatRange({ min: 0.001, max: 1.9952624, factor: 0.1813854 }), unit: " dB", value_to_string: ReactPlug.Formatters.v2s_f32_gain_to_db(2), id: "gain" }), 
    reversed: new ReactPlug.Parameters.FloatParam({ name: "Gain", defaultValue: 1, range: new ReactPlug.Ranges.ReversedFloatRange(new ReactPlug.Ranges.LinearFloatRange({ min: 0, max: 1 })), id: "reversed" }), 
    bool_test: new ReactPlug.Parameters.BoolParam({ name: "Bool Test", defaultValue: false, id: "bool_test" }), 
    int_test: new ReactPlug.Parameters.IntParam({ name: "Int Test", defaultValue: 0, range: new ReactPlug.Ranges.LinearIntRange({ min: 0, max: 10 }), id: "int_test" }), 
    wave_shape: new ReactPlug.Parameters.EnumParam({ name: "Wave Shape", defaultValue: "Sine", variants: { "Sine": "Sine Wave", "Square": "Square Wave", "WhiteNoise": "White Noise", "Dirac": "Dirac" }, id: "wave_shape" })
});
