use nih_plug::prelude::*;
use react_plug::prelude::*;

define_params! {
    ExampleParams {
        gain: FloatParam {
            name: "Gain",
            default_value: util::db_to_gain(0.0),
            range: FloatRange::Skewed {
                min: util::db_to_gain(-60.0),
                max: util::db_to_gain(6.0),
                factor: FloatRange::gain_skew_factor(-60.0, 6.0),
            },
            smoother: SmoothingStyle::Logarithmic(50.0),
            unit: " dB",
            value_to_string: formatters::v2s_f32_gain_to_db(2),
            string_to_value: formatters::s2v_f32_gain_to_db(),
        },
        reversed: FloatParam {
            name: "Gain",
            default_value: util::db_to_gain(0.0),
            range: FloatRange::Reversed (
                &FloatRange::Linear {
                    min: 0.0,
                    max: 1.0
                }
            ),
        },
        bool_test: BoolParam {
            name: "Bool Test",
            default_value: false
        },
        int_test: IntParam {
            name: "Int Test",
            default_value: 0,
            range: IntRange::Linear { min: 0, max: 10 }
        },
        wave_shape: EnumParam {
            name: "Wave Shape",
            default_value: Sine,
            variants: Waveform {
                Sine: "Sine Wave",
                Square: "Square Wave",
                WhiteNoise: "White Noise",
                Dirac
            }
        }
    }
}
