/// The `not_unit_and` function checks if the input `x` is of type unit `()`.
///
/// If `x` is of type unit, it returns the `default` value provided.
/// If `x` is not of type unit, it applies the `callback` function to `x` and returns the result.
///
/// # Arguments
///
/// * `x` - The value to be checked. It can be of any type.
/// * `default` - The value to return if `x` is of type unit `()`.
/// * `callback` - A function that will be applied to `x` if `x` is not of type unit `()`.
fn not_unit_and(x, default_x, callback) {
  if type_of(x) == "()" {
    return default_x;
  }
  callback.call(x)
}

/// Returns `true` if `x` is unit type
fn is_unit(x) {
  type_of(x) == "()"
}
