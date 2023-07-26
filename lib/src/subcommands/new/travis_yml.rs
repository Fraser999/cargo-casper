use super::{common, Error, Options};

const FILENAME: &str = ".travis.yml";
const CONTENTS: &str = r#"language: rust
script:
  - make prepare
  - make check-lint
  - make test
"#;

pub fn create(options: &Options) -> Result<(), Error> {
    common::write_file(options.root_path.join(FILENAME), CONTENTS)
}
