use pdifflib::params::{LoggingSchema as _, ParameterSchema as _};

#[derive(pdifflib::ParameterSchema)]
#[allow(dead_code)]
struct Config {
    #[parameter(
        description = "Signed offset",
        minimum = -2,
        exclusive_minimum = -3.0,
        allowed_values = [-1.0, 0, 1.0],
        default = -1.0
    )]
    offset: f64,
    #[parameter(description = "Grid size", minimum = 3, default = 128)]
    size: usize,
    #[parameter(description = "Enabled")]
    enabled: bool,
    #[parameter(description = "Output name")]
    output: String,
}

#[derive(pdifflib::LoggingSchema)]
#[allow(dead_code)]
struct Log {
    #[logging(description = "Time")]
    time: f64,
    #[logging(description = "Finished")]
    finished: bool,
    #[logging(description = "Message")]
    message: String,
}

#[derive(pdifflib_derive::Field)]
#[field(name = "temperature")]
struct GenericField<const N: usize> {
    f: ndarray::Array2<f64>,
}

#[test]
fn derives_signed_numeric_schema() {
    let parameters = Config::parameters();
    assert_eq!(parameters[0].minimum, Some(-2.0));
    assert_eq!(parameters[0].exclusive_minimum, Some(-3.0));
    assert_eq!(parameters[0].allowed_values, Some(&[-1.0, 0.0, 1.0][..]));
    assert_eq!(parameters[0].default, Some(-1.0));
    assert_eq!(parameters[1].value_type, "integer");
    assert_eq!(parameters[1].default, Some(128.0));
    assert_eq!(parameters[2].value_type, "boolean");
    assert_eq!(parameters[3].value_type, "string");
    assert!(Config::schema_json("config.toml")
        .unwrap()
        .contains("\"default\": -1"));
}

#[test]
fn derives_logging_schema() {
    let columns = Log::columns();
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0].value_type, "number");
    assert_eq!(columns[1].value_type, "boolean");
    assert_eq!(columns[2].value_type, "string");
}

#[test]
fn field_derive_supports_generics_without_trait_import() {
    let field = GenericField::<3> {
        f: ndarray::Array2::zeros((3, 3)),
    };
    assert_eq!(pdifflib::field::Field::name(&field), "temperature");
    assert_eq!(pdifflib::field::Field::get_f(&field).dim(), (3, 3));
}
