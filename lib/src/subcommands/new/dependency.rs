use super::Options;

/// Used to hold the information about the Casper dependencies which will be required by the
/// generated Cargo.toml files.
#[derive(Debug)]
pub struct Dependency {
    name: &'static str,
    version: &'static str,
}

impl Dependency {
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Dependency { name, version }
    }

    pub fn display_with_features(
        &self,
        options: &Options,
        default_features: bool,
        features: Vec<&str>,
    ) -> String {
        let version = if options.casper_overrides.is_some() {
            "*"
        } else {
            self.version
        };

        if default_features && features.is_empty() {
            return format!("{} = \"{}\"\n", self.name, version);
        }

        let mut output = format!(r#"{} = {{ version = "{}""#, self.name, version);

        if !default_features {
            output = format!("{}, default-features = false", output);
        }

        if !features.is_empty() {
            output = format!("{}, features = {:?}", output, features);
        }

        format!("{} }}\n", output)
    }

    #[cfg(test)]
    pub fn name(&self) -> &str {
        self.name
    }

    #[cfg(test)]
    pub fn version(&self) -> &str {
        self.version
    }
}
