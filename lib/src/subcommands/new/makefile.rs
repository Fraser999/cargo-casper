use super::{common, Error, Options};

const FILENAME: &str = "Makefile";
const MAKEFILE_CONTENTS: &str = include_str!("../../../resources/Makefile.in");

pub fn create(options: &Options) -> Result<(), Error> {
    common::write_file(options.root_path.join(FILENAME), MAKEFILE_CONTENTS)
}
