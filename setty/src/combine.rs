/////////////////////////////////////////////////////////////////////////////////////////

pub trait Combine {
    fn merge(lhs: &mut serde_json::Value, rhs: serde_json::Value) {
        // Default is `replace`
        *lhs = rhs;
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl<T> Combine for Vec<T> {
    fn merge(lhs: &mut serde_json::Value, rhs: serde_json::Value) {
        let mut rhs = match rhs {
            serde_json::Value::Array(v) => v,
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
    fn merge(lhs: &mut serde_json::Value, rhs: serde_json::Value) {
        let rhs = match rhs {
            serde_json::Value::Object(v) => v,
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
