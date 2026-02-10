use crate::Value;

/////////////////////////////////////////////////////////////////////////////////////////

/// Trait describing how to merge two JSON `Value`s corresponding to a particular
/// Rust type. It is used when combining values across multiple [`crate::source::Source`]s.
///
/// The implementation receives a mutable left-hand value `lhs` and a
/// right-hand `rhs` to merge into it. The default behaviour is to
/// replace the left value with the right one; collection-like types
/// often provide more sophisticated merging behaviour.
pub trait Combine {
    /// Merge `rhs` into `lhs`.
    fn merge(lhs: &mut Value, rhs: Value) {
        // Default is `replace`
        *lhs = rhs;
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl<T> Combine for Vec<T> {
    fn merge(lhs: &mut Value, rhs: Value) {
        let mut rhs = match rhs {
            Value::Array(v) => v,
            _ => {
                *lhs = rhs;
                return;
            }
        };
        let Some(lhs) = lhs.as_array_mut() else {
            *lhs = rhs.into();
            return;
        };

        lhs.append(&mut rhs);
    }
}

impl<T> Combine for std::collections::BTreeMap<String, T> {
    fn merge(lhs: &mut Value, rhs: Value) {
        let rhs = match rhs {
            Value::Object(v) => v,
            _ => {
                *lhs = rhs;
                return;
            }
        };
        let Some(lhs) = lhs.as_object_mut() else {
            *lhs = rhs.into();
            return;
        };

        for (k, v) in rhs {
            lhs.insert(k, v);
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl Combine for std::path::PathBuf {}

/////////////////////////////////////////////////////////////////////////////////////////
