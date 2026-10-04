//! Machine-readable descriptions of simulation input parameters.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameter {
    pub name: &'static str,
    pub value_type: &'static str,
    pub description: &'static str,
    pub minimum: Option<f64>,
    pub exclusive_minimum: Option<f64>,
    pub allowed_values: Option<&'static [f64]>,
    pub default: Option<f64>,
}

pub trait ParameterSchema {
    fn parameters() -> &'static [Parameter];

    fn schema_json(config_file: &str) -> String {
        input_schema_json(Self::parameters(), config_file)
    }
}

pub type LogColumn = Parameter;

pub trait LoggingSchema {
    fn columns() -> &'static [LogColumn];

    fn schema_json(log_file: &str) -> String {
        logging_schema_json(Self::columns(), log_file)
    }
}

pub fn input_schema_json(parameters: &[Parameter], config_file: &str) -> String {
    let mut json = String::from("{\n  \"config_format\": \"toml\",\n  \"config_file\": ");
    push_json_string(&mut json, config_file);
    json.push_str(",\n  \"parameters\": {");

    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("\n    ");
        push_json_string(&mut json, parameter.name);
        json.push_str(": {\"type\": ");
        push_json_string(&mut json, parameter.value_type);
        json.push_str(", \"description\": ");
        push_json_string(&mut json, parameter.description);
        if let Some(minimum) = parameter.minimum {
            json.push_str(", \"minimum\": ");
            json.push_str(&minimum.to_string());
        }
        if let Some(minimum) = parameter.exclusive_minimum {
            json.push_str(", \"exclusive_minimum\": ");
            json.push_str(&minimum.to_string());
        }
        if let Some(values) = parameter.allowed_values {
            json.push_str(", \"allowed_values\": [");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    json.push_str(", ");
                }
                json.push_str(&value.to_string());
            }
            json.push(']');
        }
        if let Some(default) = parameter.default {
            json.push_str(", \"default\": ");
            json.push_str(&default.to_string());
        }
        json.push('}');
    }

    json.push_str("\n  }\n}");
    json
}

pub fn logging_schema_json(columns: &[LogColumn], log_file: &str) -> String {
    let mut json = String::from("{\n  \"format\": \"csv\",\n  \"file\": ");
    push_json_string(&mut json, log_file);
    json.push_str(",\n  \"columns\": {");

    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("\n    ");
        push_json_string(&mut json, column.name);
        json.push_str(": {\"type\": ");
        push_json_string(&mut json, column.value_type);
        json.push_str(", \"description\": ");
        push_json_string(&mut json, column.description);
        json.push('}');
    }

    json.push_str("\n  }\n}");
    json
}

fn push_json_string(json: &mut String, value: &str) {
    json.push('"');
    for character in value.chars() {
        match character {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            character if character <= '\u{1f}' => {
                use std::fmt::Write;
                write!(json, "\\u{:04x}", character as u32).unwrap();
            }
            character => json.push(character),
        }
    }
    json.push('"');
}

#[cfg(test)]
mod tests {
    use super::{input_schema_json, logging_schema_json, Parameter};

    #[test]
    fn produces_valid_schema_with_escaped_descriptions() {
        let schema = input_schema_json(
            &[Parameter {
                name: "ra",
                value_type: "number",
                description: "A \"quoted\" value",
                minimum: Some(0.0),
                exclusive_minimum: None,
                allowed_values: None,
                default: None,
            }],
            "config.toml",
        );

        assert_eq!(
            schema,
            "{\n  \"config_format\": \"toml\",\n  \"config_file\": \"config.toml\",\n  \"parameters\": {\n    \"ra\": {\"type\": \"number\", \"description\": \"A \\\"quoted\\\" value\", \"minimum\": 0}\n  }\n}"
        );
    }

    #[test]
    fn produces_csv_log_schema() {
        let schema = logging_schema_json(
            &[Parameter {
                name: "nu",
                value_type: "number",
                description: "Nusselt number",
                minimum: None,
                exclusive_minimum: None,
                allowed_values: None,
                default: None,
            }],
            "foo.csv",
        );

        assert_eq!(
            schema,
            "{\n  \"format\": \"csv\",\n  \"file\": \"foo.csv\",\n  \"columns\": {\n    \"nu\": {\"type\": \"number\", \"description\": \"Nusselt number\"}\n  }\n}"
        );
    }

    #[test]
    fn includes_allowed_values_in_input_schema() {
        let schema = input_schema_json(
            &[Parameter {
                name: "concentration_scheme",
                value_type: "number",
                description: "Concentration scheme",
                minimum: None,
                exclusive_minimum: None,
                allowed_values: Some(&[0.0, 1.0]),
                default: None,
            }],
            "config.toml",
        );

        assert!(schema.contains("\"allowed_values\": [0, 1]"));
    }
}
