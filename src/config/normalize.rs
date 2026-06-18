pub(super) fn normalize_string_vec(values: &mut Vec<String>) {
    values.iter_mut().for_each(|value| {
        *value = value.trim().to_string();
    });
    values.retain(|value| !value.is_empty());
    values.sort();
    values.dedup();
}

pub(super) fn take_non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
