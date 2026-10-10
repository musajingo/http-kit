use axum::extract::rejection::JsonRejection;
use serde_json::json;
use valid8r::{ValidationErrors, ValidationErrorsKind};

/// Extracts field-level details from a JSON deserialization error.
pub fn json_data_error_details(rejection: &JsonRejection) -> Option<serde_json::Value> {
    let JsonRejection::JsonDataError(rejection) = rejection else {
        return None;
    };

    let mut source = std::error::Error::source(rejection);
    while let Some(err) = source {
        if let Some(err) = err.downcast_ref::<serde_path_to_error::Error<serde_json::Error>>() {
            let field = err.path().to_string();
            if field.is_empty() || field == "." {
                return None;
            }

            // serde_json appends " at line L column C" to the message.
            let message = err.inner().to_string();
            let message = message
                .rsplit_once(" at line ")
                .map_or(message.as_str(), |(message, _)| message);
            let message = drop_submitted_value(message);

            let mut details = serde_json::Map::new();
            details.insert(
                field,
                json!([{ "code": "invalid", "message": message, "params": {} }]),
            );
            return Some(serde_json::Value::Object(details));
        }
        source = err.source();
    }

    None
}

/// Serde messages like `invalid type: integer `123456`, expected a string`
/// echo the submitted value, which may be a credential. Keep only what was
/// expected, and allowlist the shapes we know are value-free: anything else
/// (e.g. a custom Deserialize impl's message) may embed the input.
fn drop_submitted_value(message: &str) -> String {
    // `unknown field` / `duplicate field` echo client-sent key names, which
    // are also input; only `missing field` names come from our own schema.
    for prefix in [
        "invalid type",
        "invalid value",
        "invalid length",
        "unknown variant",
        "unknown field",
        "duplicate field",
    ] {
        if message.starts_with(prefix) {
            return match message.rsplit_once(", expected ") {
                Some((_, expected)) => format!("{prefix}, expected {expected}"),
                None => prefix.to_string(),
            };
        }
    }
    if message.starts_with("missing field") {
        return message.to_string();
    }
    "could not deserialize value".to_string()
}

/// Converts nested validation errors into API error details.
pub fn validation_errors_to_json(validation_errors: &ValidationErrors) -> serde_json::Value {
    let mut errors = serde_json::Map::new();

    for (field, field_errors) in validation_errors.errors() {
        // `valid8r` keys errors by the raw Rust field ident; the API field names
        // are camelCase (`#[serde(rename_all = "camelCase")]`), so map the key.
        let key = to_camel_case(field);
        match field_errors {
            ValidationErrorsKind::Field(field_errors) => {
                let details: Vec<serde_json::Value> = field_errors
                    .iter()
                    .map(|err| {
                        json!({
                            "code": err.code.to_string(),
                            "message": err.message.as_ref().map(|msg| msg.to_string()).unwrap_or_default(),
                            "params": err.params
                        })
                    })
                    .collect();

                errors.insert(key, json!(details));
            }
            ValidationErrorsKind::Struct(struct_errors) => {
                errors.insert(key, validation_errors_to_json(struct_errors));
            }
            ValidationErrorsKind::List(list_errors) => {
                // Keyed by the item's index so clients know which entry failed.
                let items: serde_json::Map<String, serde_json::Value> = list_errors
                    .iter()
                    .map(|(index, item_errors)| {
                        (index.to_string(), validation_errors_to_json(item_errors))
                    })
                    .collect();

                errors.insert(key, serde_json::Value::Object(items));
            }
        }
    }

    serde_json::Value::Object(errors)
}

/// Returns field paths and error codes for logging without submitted values.
pub fn rejected_fields(validation_errors: &ValidationErrors) -> Vec<String> {
    let mut out = Vec::new();
    collect_rejected(validation_errors, "", &mut out);
    out.sort();
    out
}

fn collect_rejected(validation_errors: &ValidationErrors, prefix: &str, out: &mut Vec<String>) {
    for (field, field_errors) in validation_errors.errors() {
        let path = format!("{prefix}{field}");
        match field_errors {
            ValidationErrorsKind::Field(field_errors) => {
                out.extend(
                    field_errors
                        .iter()
                        .map(|err| format!("{path}:{}", err.code)),
                );
            }
            ValidationErrorsKind::Struct(struct_errors) => {
                collect_rejected(struct_errors, &format!("{path}."), out);
            }
            ValidationErrorsKind::List(list_errors) => {
                for (index, item_errors) in list_errors {
                    collect_rejected(item_errors, &format!("{path}[{index}]."), out);
                }
            }
        }
    }
}

fn to_camel_case(field: &str) -> String {
    // Schema-level errors use the sentinel `__all__`; leave it intact rather
    // than mangling it to "All".
    if field == "__all__" {
        return field.to_string();
    }
    let mut out = String::with_capacity(field.len());
    let mut capitalize_next = false;
    for (i, ch) in field.chars().enumerate() {
        if ch == '_' {
            capitalize_next = i != 0;
            continue;
        }
        if capitalize_next {
            out.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;
    use valid8r::Validate;

    #[derive(Debug, Validate, Deserialize)]
    struct TestItem {
        #[validate(length(min = 1, message = "ID cannot be empty"))]
        id: String,

        #[validate(range(min = 0, max = 100, message = "Value out of range"))]
        value: i32,
    }

    #[derive(Debug, Validate, Deserialize)]
    struct TestNestedStruct {
        #[validate(email(message = "Invalid email format"))]
        email: String,
        #[validate(nested)]
        item: TestItem,
    }

    #[derive(Debug, Validate, Deserialize)]
    struct TestListStruct {
        #[validate(length(min = 3, message = "Name too short"))]
        name: String,
        #[validate(nested)]
        items: Vec<TestItem>,
    }

    #[test]
    fn test_validation_errors_to_json_simple_field_errors() {
        let item = TestItem {
            id: "".to_string(), // Fails length validation
            value: 101,         // Fails range validation
        };

        let validation_result = item.validate();

        assert!(validation_result.is_err(), "Expected validation errors");

        let validation_errors = validation_result.unwrap_err();

        let json_errors = validation_errors_to_json(&validation_errors);

        assert_eq!(
            json_errors,
            json!({
                "id": [
                    {
                        "code": "length",
                        "message": "ID cannot be empty",
                        "params": {
                            "min": 1,
                            "value": ""
                        }
                    }
                ],
                "value": [
                    {
                        "code": "range",
                        "message": "Value out of range",
                        "params": {
                            "min": 0,
                            "max": 100,
                            "value": 101
                        }
                    }
                ]
            })
        );
    }

    #[test]
    fn test_validation_errors_to_json_nested_struct_errors() {
        let nested_struct = TestNestedStruct {
            email: "not-an-email".to_string(), // Fails email validation
            item: TestItem {
                id: "".to_string(), // Fails length validation in nested struct
                value: 50,          // Valid
            },
        };

        let validation_result = nested_struct.validate();

        assert!(validation_result.is_err());

        let validation_error = validation_result.unwrap_err();

        let json_errors = validation_errors_to_json(&validation_error);

        assert_eq!(
            json_errors,
            json!({
                "email": [
                    {
                        "message": "Invalid email format",
                        "params": {
                        "value": "not-an-email"
                        },
                        "code": "email"
                    }
                ],
                "item": {
                    "id": [
                        {
                        "message": "ID cannot be empty",
                        "params": {
                            "min": 1,
                            "value": ""
                        },
                        "code": "length"
                        }
                    ]
                }
            })
        );
    }

    #[test]
    fn test_validation_errors_to_json_list_errors() {
        let list_struct = TestListStruct {
            name: "ab".to_string(), // Fails length validation
            items: vec![
                TestItem {
                    id: "valid_id_1".to_string(),
                    value: 10, // Valid
                },
                TestItem {
                    id: "".to_string(), // Fails length validation
                    value: 101,         // Fails range validation
                },
                TestItem {
                    id: "valid_id_2".to_string(),
                    value: 20, // Valid
                },
            ],
        };

        let validation_result = list_struct.validate();

        assert!(validation_result.is_err());

        let validation_error = validation_result.unwrap_err();

        let json_errors = validation_errors_to_json(&validation_error);

        assert_eq!(
            json_errors,
            json!({
                "name": [
                    {
                        "message": "Name too short",
                        "params": {
                            "min": 3,
                            "value": "ab"
                        },
                        "code": "length"
                    }
                ],
                // Keyed by index: the second item (index 1) is the invalid one.
                "items": {
                    "1": {
                        "id": [
                            {
                                "message": "ID cannot be empty",
                                "params": {
                                    "min": 1,
                                    "value": ""
                                },
                                "code": "length"
                            }
                        ],
                        "value": [
                            {
                                "message": "Value out of range",
                                "params": {
                                    "max": 100,
                                    "min": 0,
                                    "value": 101
                                },
                                "code": "range"
                            }
                        ]
                    }
                },
            })
        );
    }

    #[test]
    fn test_validation_errors_to_json_no_errors() {
        let item = TestItem {
            id: "valid_id".to_string(),
            value: 50,
        };

        let validation_result = item.validate();
        assert!(validation_result.is_ok(), "Expected no validation errors");
    }

    #[test]
    fn test_validation_errors_to_json_field_error_no_message() {
        #[derive(Debug, Validate, Deserialize)]
        struct TestNoMessage {
            #[validate(required)]
            // 'required' validator doesn't have a default message in the error itself
            field: Option<String>,
        }

        let item = TestNoMessage { field: None };
        let validation_result = item.validate();
        assert!(validation_result.is_err());

        if let Err(errors) = validation_result {
            let json_errors = validation_errors_to_json(&errors);
            assert_eq!(json_errors["field"][0]["message"], ""); // Expecting empty string as per current implementation
            assert_eq!(json_errors["field"][0]["code"], "required");
        } else {
            panic!("Expected validation errors");
        }
    }

    #[test]
    fn test_validation_errors_to_json_camel_cases_multiword_keys() {
        #[derive(Debug, Validate, Deserialize)]
        struct Inner {
            #[validate(length(min = 1, message = "required"))]
            id: String,
        }
        #[derive(Debug, Validate, Deserialize)]
        struct Outer {
            #[validate(length(min = 2, message = "too short"))]
            first_name: String,
            #[validate(nested)]
            home_address: Inner,
        }

        let errors = Outer {
            first_name: String::new(),
            home_address: Inner { id: String::new() },
        }
        .validate()
        .unwrap_err();

        let json = validation_errors_to_json(&errors);

        assert!(
            json.get("firstName").is_some(),
            "expected camelCase key, got {json}"
        );
        assert!(json.get("first_name").is_none());
        assert!(
            json["homeAddress"].get("id").is_some(),
            "nested struct key should camelCase to homeAddress"
        );
    }

    #[test]
    fn to_camel_case_handles_common_shapes() {
        assert_eq!(to_camel_case("first_name"), "firstName");
        assert_eq!(to_camel_case("max_workload"), "maxWorkload");
        assert_eq!(to_camel_case("name"), "name");
        assert_eq!(to_camel_case("iso2"), "iso2");
        assert_eq!(to_camel_case("assigned_employee_id"), "assignedEmployeeId");
    }
}
