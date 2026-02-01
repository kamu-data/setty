use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use figment2::Provider as _;

use crate::{Value, format::Format};

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Config<Cfg, Fmt> {
    paths: Vec<PathBuf>,
    env_var_prefix: Option<String>,
    _t_cfg: PhantomData<Cfg>,
    _t_prov: PhantomData<Fmt>,
}

/////////////////////////////////////////////////////////////////////////////////////////

impl<Cfg, Fmt> Default for Config<Cfg, Fmt> {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            env_var_prefix: None,
            _t_cfg: PhantomData,
            _t_prov: PhantomData,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(all(feature = "derive-deserialize", feature = "derive-serialize"))]
impl<Cfg, Fmt> Config<Cfg, Fmt>
where
    Cfg: Default,
    Cfg: serde::Deserialize<'static>,
    Cfg: serde::Serialize,
    Fmt: Format,
    Fmt: figment2::providers::Format,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.paths.push(path.into());
        self
    }

    pub fn with_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        for p in paths.into_iter() {
            self.paths.push(p.into());
        }
        self
    }

    pub fn with_env_var_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_var_prefix = Some(prefix.into());
        self
    }

    /// Deserializes the marged config into a struct
    pub fn extract(&self) -> Result<Cfg, ReadError> {
        self.to_figment().extract().map_err(Into::into)
    }

    /// Returns raw merged data
    #[cfg(not(feature = "derive-jsonschema"))]
    pub fn data(&self, with_defaults: bool) -> Result<figment2::value::Dict, ReadError> {
        if with_defaults {
            panic!("Merging with default currently requires `setty/derive-jsonschema` feature")
        }
        let fig = self.to_figment();
        let mut data = fig.data()?;
        let value = data.remove(&figment2::Profile::default()).unwrap();
        Ok(value)
    }

    /// Returns value under specified path
    #[cfg(not(feature = "derive-jsonschema"))]
    pub fn get_value(&self, path: &str, with_defaults: bool) -> Result<Option<Value>, ReadError> {
        let data = self.data(with_defaults)?;
        Ok(Self::find_value(path, data.into()))
    }

    /// Sets the value under specified path creating new or merging it into existing config file
    pub fn set_value(
        &self,
        path: &str,
        value: impl Into<Value>,
        in_config_path: impl AsRef<Path>,
    ) -> Result<(), WriteError> {
        self.set_value_impl(path, value.into(), in_config_path.as_ref())
    }

    fn set_value_impl(
        &self,
        path: &str,
        mut value: Value,
        in_config_path: &Path,
    ) -> Result<(), WriteError> {
        use figment2::value::{Dict, Tag, Value};
        // Nest value under the path
        for segment in path.rsplit('.') {
            value = Value::Dict(Tag::Default, Dict::from([(segment.to_string(), value)]));
        }

        // Read config merged with the new value to validate
        {
            let fig = self
                .to_figment()
                .admerge(figment2::providers::Serialized::defaults(&value));
            fig.extract::<Cfg>()?;
        }

        let content = if !in_config_path.is_file() {
            if let Some(dir) = in_config_path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            Fmt::serialize(&value).map_err(|e| figment2::Error::from(e.to_string()))?
        } else {
            // Read target config merged with the new value
            let fig = Self::new()
                .with_path(in_config_path)
                .to_figment()
                .admerge(figment2::providers::Serialized::defaults(&value));

            let mut merged = fig.data()?;
            let merged = merged.remove(&figment2::Profile::default()).unwrap();

            Fmt::serialize(&merged).map_err(|e| figment2::Error::from(e.to_string()))?
        };

        std::fs::write(in_config_path, content)?;
        Ok(())
    }

    // TODO: Validate new config before writing?
    pub fn unset_value(
        &self,
        path: &str,
        in_config_path: &Path,
    ) -> Result<Option<Value>, WriteError> {
        let data = std::fs::read(in_config_path)?;
        let data = str::from_utf8(&data).unwrap();
        let mut value: Value =
            Fmt::deserialize(data).map_err(|e| figment2::Error::from(e.to_string()))?;

        let prev_value = Self::unset_rec(path, as_dict_mut(&mut value).unwrap());

        let new_data = Fmt::serialize(&value).map_err(|e| figment2::Error::from(e.to_string()))?;

        std::fs::write(in_config_path, new_data)?;

        Ok(prev_value)
    }

    fn unset_rec(path: &str, obj: &mut figment2::value::Dict) -> Option<Value> {
        if let Some((head, tail)) = path.split_once('.') {
            if let Some(child) = obj.get_mut(head).and_then(as_dict_mut) {
                Self::unset_rec(tail, child)
            } else {
                None
            }
        } else {
            obj.remove(path)
        }
    }

    /// Escape hatch to get the merged [`figment2::Figment`] object
    pub fn to_figment(&self) -> figment2::Figment {
        let mut fig = figment2::Figment::new();

        for p in &self.paths {
            fig = fig.admerge(Fmt::file(p).search(false).required(true));
        }

        fig = if let Some(env_var_prefix) = &self.env_var_prefix {
            fig.admerge(
                figment2::providers::Env::prefixed(env_var_prefix)
                    .split("__")
                    .lowercase(false),
            )
        } else {
            fig
        };

        fig
    }

    fn find_value(path: &str, data: Value) -> Option<Value> {
        let mut current = data;

        for segment in path.split(".") {
            let mut dict = match current {
                Value::Dict(_, value) => value,
                _ => return None,
            };
            let child = dict.remove(segment)?;
            current = child;
        }

        Some(current)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "derive-jsonschema")]
impl<Cfg, Fmt> Config<Cfg, Fmt>
where
    Cfg: Default,
    Cfg: serde::Deserialize<'static>,
    Cfg: serde::Serialize,
    Cfg: schemars::JsonSchema,
    Fmt: Format,
    Fmt: figment2::providers::Format,
{
    /// Returns raw merged data
    #[cfg(feature = "derive-jsonschema")]
    pub fn data(&self, with_defaults: bool) -> Result<figment2::value::Dict, ReadError> {
        let fig = self.to_figment();
        let mut data = fig.data()?;
        let value = data.remove(&figment2::Profile::default()).unwrap();

        if !with_defaults {
            return Ok(value);
        }

        // Transcode value into JSON to work in one domain
        let mut value: serde_json::Value = {
            let json = serde_json::to_string(&value).unwrap();
            serde_json::from_str(&json).unwrap()
        };

        // Get schema that has all variants and defaults
        let schema = self.json_schema().to_value();
        let null = serde_json::Value::Null;
        let defs = schema.get("$defs").unwrap_or(&null);

        // Begin the merge-aroo!
        crate::merge_with_defaults::merge_with_defaults(
            schema["title"].as_str().unwrap(),
            &mut value,
            &schema,
            defs,
        );

        // Transcode back ... ugh, I know
        let value: Value = {
            let json = serde_json::to_string(&value).unwrap();
            serde_json::from_str(&json).unwrap()
        };

        Ok(value.into_dict().unwrap())
    }

    /// Returns value under specified path
    #[cfg(feature = "derive-jsonschema")]
    pub fn get_value(&self, path: &str, with_defaults: bool) -> Result<Option<Value>, ReadError> {
        let data = self.data(with_defaults)?;
        Ok(Self::find_value(path, data.into()))
    }

    /// Returns JSON Schema describing the config type
    pub fn json_schema(&self) -> schemars::Schema {
        schemars::schema_for!(Cfg)
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
        obj: &serde_json::Map<String, serde_json::Value>,
        defs: &serde_json::Map<String, serde_json::Value>,
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

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ReadError(Box<figment2::error::Error>);

impl ReadError {
    pub fn kind(&self) -> &figment2::error::Kind {
        &self.0.kind
    }

    pub fn is_missing_field(&self) -> bool {
        self.0.missing()
    }
}

impl From<figment2::Error> for ReadError {
    fn from(value: figment2::Error) -> Self {
        Self(Box::new(value))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum WriteError {
    Read(#[from] ReadError),
    Io(#[from] std::io::Error),
}

impl From<figment2::Error> for WriteError {
    fn from(value: figment2::Error) -> Self {
        Self::Read(value.into())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

fn as_dict_mut(value: &mut Value) -> Option<&mut figment2::value::Dict> {
    match value {
        Value::Dict(_tag, dict) => Some(dict),
        _ => None,
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
