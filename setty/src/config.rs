#![allow(unused)]

use std::{marker::PhantomData, path::Path, rc::Rc};

use crate::Value;
use crate::combine::Combine;
use crate::errors::{ReadError, WriteError};
use crate::format::Format;
use crate::source::Source;

/////////////////////////////////////////////////////////////////////////////////////////

pub type OnDeprecatedClb =
    dyn Fn(&[&str], /* reason */ Option<&str>, /* since */ Option<&str>) + 'static;

/////////////////////////////////////////////////////////////////////////////////////////

/// Builder that collects configuration `Source`s and merges them.
///
/// `Config<Cfg>` is generic over a target configuration type `Cfg`. When
/// `Cfg` implements `serde::Deserialize` and `Combine`, `Config` can
/// deserialize the merged configuration into the typed value using
/// `extract()`.
pub struct Config<Cfg> {
    sources: Vec<Box<dyn Source>>,
    deprecation_clb: Option<Box<OnDeprecatedClb>>,
    _p: PhantomData<Cfg>,
}

/////////////////////////////////////////////////////////////////////////////////////////

impl<Cfg> Default for Config<Cfg> {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            deprecation_clb: None,
            _p: PhantomData,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "derive-deserialize")]
impl<Cfg> Config<Cfg>
where
    Cfg: serde::de::DeserializeOwned,
    Cfg: Combine,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_source(mut self, source: impl Source + 'static) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    pub fn with_sources<I, S>(mut self, sources: I) -> Self
    where
        S: Source + 'static,
        I: IntoIterator<Item = S>,
    {
        for source in sources {
            self = self.with_source(source);
        }
        self
    }

    /// Deserializes the marged config into the config type
    #[cfg(not(feature = "derive-jsonschema"))]
    pub fn extract(&self) -> Result<Cfg, ReadError> {
        let value = self.data_combined(None)?;
        serde_json::from_value(value).map_err(|e| ReadError::Serde(e.into()))
    }

    fn data_combined(
        &self,
        extra_source: Option<&dyn crate::source::Source>,
    ) -> Result<Value, ReadError> {
        let mut combined = Value::Object(Default::default());

        for source in self.sources.iter().map(|b| b.as_ref()).chain(extra_source) {
            let Some(new) = source.load()? else {
                continue;
            };

            combined = if combined.as_object().unwrap().is_empty() {
                new
            } else {
                Cfg::merge(&mut combined, new);
                combined
            };
        }

        Ok(combined)
    }

    /// Returns raw merged data
    #[cfg(not(feature = "derive-jsonschema"))]
    pub fn data(&self, with_defaults: bool) -> Result<Value, ReadError> {
        if with_defaults {
            panic!("Merging with default currently requires `setty/derive-jsonschema` feature")
        }
        self.data_combined(None)
    }

    /// Returns value under specified path
    #[cfg(not(feature = "derive-jsonschema"))]
    pub fn get_value(&self, path: &str, with_defaults: bool) -> Result<Option<Value>, ReadError> {
        let data = self.data(with_defaults)?;
        Ok(Self::find_value(path, data.into()))
    }

    /// Sets the value under specified path creating new or merging it into existing config file
    pub fn set_value<Fmt>(
        &self,
        path: &str,
        value: impl Into<Value>,
        in_config_path: impl AsRef<Path>,
    ) -> Result<(), WriteError>
    where
        Fmt: 'static,
        Fmt: Format,
    {
        self.set_value_impl::<Fmt>(path, value.into(), in_config_path.as_ref())
    }

    fn set_value_impl<Fmt>(
        &self,
        path: &str,
        mut value: Value,
        in_config_path: &Path,
    ) -> Result<(), WriteError>
    where
        Fmt: 'static,
        Fmt: Format,
    {
        // Nest value under the path
        for segment in path.rsplit('.') {
            let mut map = serde_json::Map::new();
            map.insert(segment.to_string(), value);
            value = map.into();
        }

        // Deserialize config merged with new values to validate before writing it to disk
        // TODO: Too much cloning
        let data = self.data_combined(Some(&value.clone()))?;
        serde_json::from_value::<Cfg>(value.clone()).map_err(|e| ReadError::Serde(e.into()))?;

        let content = if !in_config_path.is_file() {
            if let Some(dir) = in_config_path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            Fmt::serialize(&value).unwrap()
        } else {
            // Read target config merged with the new value
            let merged = Self::new()
                .with_source(crate::source::File::<Fmt>::new(in_config_path))
                .with_source(value.clone())
                .data_combined(None)?;

            Fmt::serialize(&merged).unwrap()
        };

        std::fs::write(in_config_path, content)?;
        Ok(())
    }

    // TODO: Validate new config before writing?
    pub fn unset_value<Fmt>(
        &self,
        path: &str,
        in_config_path: &Path,
    ) -> Result<Option<Value>, WriteError>
    where
        Fmt: Format,
    {
        let data = std::fs::read(in_config_path)?;
        let data = str::from_utf8(&data).unwrap();
        let mut value: Value = Fmt::deserialize(data).map_err(|e| ReadError::Serde(e.into()))?;

        let prev_value = Self::unset_rec(path, value.as_object_mut());

        let new_data = Fmt::serialize(&value).unwrap();

        std::fs::write(in_config_path, new_data)?;

        Ok(prev_value)
    }

    fn unset_rec(path: &str, obj: Option<&mut serde_json::Map<String, Value>>) -> Option<Value> {
        let obj = obj?;

        if let Some((head, tail)) = path.split_once('.') {
            Self::unset_rec(tail, obj.get_mut(head).and_then(|v| v.as_object_mut()))
        } else {
            obj.remove(path)
        }
    }

    fn find_value(path: &str, data: Value) -> Option<Value> {
        let mut current = data;

        for segment in path.split(".") {
            let mut mamp = match current {
                Value::Object(v) => v,
                _ => return None,
            };
            let child = mamp.remove(segment)?;
            current = child;
        }

        Some(current)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "derive-jsonschema")]
impl<
    #[cfg(not(feature = "derive-validate"))] Cfg: serde::de::DeserializeOwned + schemars::JsonSchema + Combine,
    #[cfg(feature = "derive-validate")] Cfg: serde::de::DeserializeOwned + schemars::JsonSchema + Combine + validator::Validate,
> Config<Cfg>
{
    /// Provide a callback used to report use of deprecated fields.
    /// The callback will receive full path of the property and an optional deprecation reason.
    pub fn with_deprecation_clb(
        mut self,
        clb: impl Fn(&[&str], Option<&str>, Option<&str>) + 'static,
    ) -> Self {
        self.deprecation_clb = Some(Box::new(clb));
        self
    }

    /// Deserializes the marged config into the config type and performs deprecation checks and validation.
    pub fn extract(&self) -> Result<Cfg, ReadError> {
        let value = self.data_combined(None)?;

        // TODO: Maybe check per every source, not after combining?
        let clb = match self.deprecation_clb.as_deref() {
            Some(clb) => clb,
            None => &crate::check_deprecated::default_deprecation_clb as &OnDeprecatedClb,
        };

        let schema = self.json_schema().to_value();
        crate::check_deprecated::check_deprecated_fields(&schema, &value, clb);

        let cfg = serde_json::from_value(value).map_err(|e| ReadError::Serde(e.into()))?;

        #[cfg(feature = "derive-validate")]
        validator::Validate::validate(&cfg)?;

        Ok(cfg)
    }

    /// Returns raw merged data
    pub fn data(&self, with_defaults: bool) -> Result<Value, ReadError> {
        let mut value = self.data_combined(None)?;

        if !with_defaults {
            return Ok(value);
        }

        // Get schema that has all variants and defaults
        let schema = self.json_schema().to_value();
        let null = Value::Null;
        let defs = schema.get("$defs").unwrap_or(&null);

        // Begin the merge-aroo!
        crate::merge_with_defaults::merge_with_defaults(
            schema["title"].as_str().unwrap(),
            &mut value,
            &schema,
            defs,
        );

        Ok(value)
    }

    /// Returns value under specified path
    pub fn get_value(&self, path: &str, with_defaults: bool) -> Result<Option<Value>, ReadError> {
        let data = self.data(with_defaults)?;
        Ok(Self::find_value(path, data))
    }

    /// Returns JSON Schema describing the config type
    pub fn json_schema(&self) -> crate::schema::Schema {
        schemars::schema_for!(Cfg).into()
    }

    /// Returns Markdown describing the config type
    pub fn markdown(&self) -> String {
        let schema = self.json_schema();
        crate::markdown::schema_to_markdown(&schema)
    }

    /// Given a prefix like `some.va` would return possible completions, e.g.
    /// `some.value` and `some.validator`
    pub fn complete_path(&self, prefix: &str) -> Vec<String> {
        let mut ret = Vec::new();
        let schema = self.json_schema();
        let schema = schema.as_object().unwrap();
        let defs = schema["$defs"].as_object().unwrap();

        // TODO: PERF: This can be improved by considering prefix while traversing
        Self::all_paths_rec("", schema, defs, &mut ret);

        ret.retain(|v| v.starts_with(prefix));

        ret
    }

    fn all_paths_rec(
        path: &str,
        obj: &serde_json::Map<String, Value>,
        defs: &serde_json::Map<String, Value>,
        ret: &mut Vec<String>,
    ) {
        let Some(properties) = obj.get("properties").and_then(|v| v.as_object()) else {
            if let Some(r) = obj.get("$ref").and_then(|v| v.as_str())
                && let Some((_, name)) = r.rsplit_once('/')
            {
                let val = defs.get(name).unwrap().as_object().unwrap();
                Self::all_paths_rec(path, val, defs, ret);
            }
            if let Some(one_of) = obj.get("oneOf").and_then(|v| v.as_array()) {
                for var in one_of {
                    Self::all_paths_rec(path, var.as_object().unwrap(), defs, ret);
                }
            }
            if let Some(any_of) = obj.get("anyOf").and_then(|v| v.as_array()) {
                for var in any_of {
                    Self::all_paths_rec(path, var.as_object().unwrap(), defs, ret);
                }
            }
            return;
        };

        for (name, val) in properties {
            let ppath = if path.is_empty() {
                name.clone()
            } else {
                format!("{path}.{name}")
            };

            ret.push(ppath.clone());

            if let Some(val) = val.as_object() {
                Self::all_paths_rec(&ppath, val, defs, ret);
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
