use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("zip error")]
    Zip(#[from] zip::result::ZipError),

    #[error("json error")]
    Json(#[from] serde_json::Error),

    #[error("toml error")]
    Toml(#[from] toml::de::Error),

    #[error("template not found")]
    TemplatesNotFound,

    #[error("render rror")]
    RenderError(#[from] crate::pattern::RenderError),
}
