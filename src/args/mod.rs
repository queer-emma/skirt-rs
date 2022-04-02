use std::path::PathBuf;

use structopt::StructOpt;

use crate::{
    error::Error,
    parameters::Parameters,
};

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Render {
        /// render this template.
        template: PathBuf,

        /// render template with this parameter file. if omitted, the template
        /// will be rendered with default parameters.
        parameters: Option<PathBuf>,
    },
}

impl Args {
    pub fn run(self) -> Result<(), Error> {
        match self.command {
            Command::Render {
                template,
                parameters,
            } => {
                let template = crate::reader::from_file(&template)?;
                log::debug!("template: {:#?}", template);

                if let Some(parameters) = parameters {
                    let toml = std::fs::read_to_string(parameters)?;
                    let parameters: Parameters = toml::from_str(&toml)?;

                    log::debug!("parameters: {:#?}", parameters);

                    // todo: apply parameters
                    let pattern = template.with_parameters(&parameters)?;
                    log::debug!("pattern: {:#?}", pattern);
                }
            }
        }

        Ok(())
    }
}
