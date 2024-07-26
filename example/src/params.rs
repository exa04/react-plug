use nih_plug::prelude::*;
use react_plug::prelude::*;

rp_params! {
    ExampleParams {
        gain: FloatParam {
            name: "Gain",
            value: util::db_to_gain(0.0),
            range: FloatRange::Skewed {
                min: util::db_to_gain(-30.0),
                max: util::db_to_gain(30.0),
                factor: FloatRange::gain_skew_factor(-30.0, 30.0),
            },
            smoother: SmoothingStyle::Logarithmic(50.0),
            unit: " dB",
            value_to_string: formatters::v2s_f32_gain_to_db(2),
            string_to_value: formatters::s2v_f32_gain_to_db(),
        },
        bool_test: BoolParam {
            name: "Bool Test",
            value: false
        },
        int_test: IntParam {
            name: "Int Test",
            value: 0,
            range: IntRange::Linear { min: 0, max: 10 }
        }
    }
}