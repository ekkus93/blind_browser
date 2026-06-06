pub fn assert_json_matches_schema(
    instance: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<(), String> {
    assert_json_matches_schema_at(instance, schema, schema, "$")
}

pub fn assert_json_matches_schema_at(
    instance: &serde_json::Value,
    schema: &serde_json::Value,
    root_schema: &serde_json::Value,
    path: &str,
) -> Result<(), String> {
    let schema = resolve_schema_reference(schema, root_schema)?;

    if let Some(all_of) = schema.get("allOf").and_then(serde_json::Value::as_array) {
        for subschema in all_of {
            assert_json_matches_schema_at(instance, subschema, root_schema, path)?;
        }
    }

    if let Some(any_of) = schema.get("anyOf").and_then(serde_json::Value::as_array) {
        let mut errors = Vec::new();
        for subschema in any_of {
            match assert_json_matches_schema_at(instance, subschema, root_schema, path) {
                Ok(()) => {
                    errors.clear();
                    break;
                }
                Err(error) => errors.push(error),
            }
        }
        if !errors.is_empty() {
            return Err(format!(
                "{path}: value did not satisfy anyOf alternatives: {}",
                errors.join(" | ")
            ));
        }
    }

    if let Some(one_of) = schema.get("oneOf").and_then(serde_json::Value::as_array) {
        let mut match_count = 0;
        let mut errors = Vec::new();
        for subschema in one_of {
            match assert_json_matches_schema_at(instance, subschema, root_schema, path) {
                Ok(()) => match_count += 1,
                Err(error) => errors.push(error),
            }
        }
        if match_count != 1 {
            return Err(format!(
                "{path}: value matched {match_count} oneOf alternatives (expected exactly 1): {}",
                errors.join(" | ")
            ));
        }
    }

    if let Some(expected_const) = schema.get("const") {
        if instance != expected_const {
            return Err(format!(
                "{path}: expected const {expected_const:?}, got {instance:?}"
            ));
        }
    }

    if let Some(enum_values) = schema.get("enum").and_then(serde_json::Value::as_array) {
        if !enum_values.iter().any(|candidate| candidate == instance) {
            return Err(format!(
                "{path}: expected one of {enum_values:?}, got {instance:?}"
            ));
        }
    }

    if let Some(type_schema) = schema.get("type") {
        if !json_matches_type(instance, type_schema) {
            return Err(format!(
                "{path}: value {instance:?} did not match schema type {type_schema:?}"
            ));
        }
    }

    if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
        let Some(object) = instance.as_object() else {
            return Err(format!("{path}: required fields only apply to objects"));
        };
        for field_name in required.iter().filter_map(serde_json::Value::as_str) {
            if !object.contains_key(field_name) {
                return Err(format!("{path}: missing required field '{field_name}'"));
            }
        }
    }

    if let Some(properties) = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
    {
        let Some(object) = instance.as_object() else {
            return Err(format!("{path}: properties only apply to objects"));
        };
        let additional_properties_allowed = schema
            .get("additionalProperties")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

        for (key, value) in object {
            if let Some(property_schema) = properties.get(key) {
                assert_json_matches_schema_at(
                    value,
                    property_schema,
                    root_schema,
                    &format!("{path}/{key}"),
                )?;
            } else if !additional_properties_allowed {
                return Err(format!(
                    "{path}: unexpected property '{key}' is not allowed by the schema"
                ));
            }
        }
    }

    if let Some(items_schema) = schema.get("items") {
        let Some(array) = instance.as_array() else {
            return Err(format!("{path}: items only apply to arrays"));
        };
        for (index, item) in array.iter().enumerate() {
            assert_json_matches_schema_at(
                item,
                items_schema,
                root_schema,
                &format!("{path}/{index}"),
            )?;
        }
    }

    Ok(())
}

pub fn resolve_schema_reference<'a>(
    schema: &'a serde_json::Value,
    root_schema: &'a serde_json::Value,
) -> Result<&'a serde_json::Value, String> {
    let Some(reference) = schema.get("$ref").and_then(serde_json::Value::as_str) else {
        return Ok(schema);
    };
    let pointer = reference
        .strip_prefix('#')
        .ok_or_else(|| format!("unsupported non-local schema reference '{reference}'"))?;
    let resolved = root_schema
        .pointer(pointer)
        .ok_or_else(|| format!("failed to resolve schema reference '{reference}'"))?;
    if std::ptr::eq(resolved, schema) {
        return Ok(resolved);
    }
    resolve_schema_reference(resolved, root_schema)
}

pub fn json_matches_type(instance: &serde_json::Value, type_schema: &serde_json::Value) -> bool {
    match type_schema {
        serde_json::Value::String(kind) => json_matches_single_type(instance, kind),
        serde_json::Value::Array(kinds) => kinds.iter().any(|kind| {
            kind.as_str()
                .is_some_and(|kind| json_matches_single_type(instance, kind))
        }),
        _ => true,
    }
}

pub fn json_matches_single_type(instance: &serde_json::Value, kind: &str) -> bool {
    match kind {
        "null" => instance.is_null(),
        "boolean" => instance.is_boolean(),
        "object" => instance.is_object(),
        "array" => instance.is_array(),
        "string" => instance.is_string(),
        "number" => instance.is_number(),
        "integer" => instance
            .as_f64()
            .is_some_and(|number| number.fract().abs() < f64::EPSILON),
        _ => false,
    }
}
