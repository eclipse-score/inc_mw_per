pub mod kvs_instance;
pub mod kvs_parameters;

/// Helper function to convert `Debug`-typed value to `String`.
pub(crate) fn to_str<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}")
}
