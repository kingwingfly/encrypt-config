use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub enum ConfigError {
    #[snafu(display("The key `{}` not found in Config", key))]
    ConfigNotFound { key: String },
    #[snafu(
        display("Failed to deseriliaze encrypter from keyring."),
        context(false)
    )]
    LoadEncrypterFailed { source: serde_json::Error },
    #[snafu(
        display(
            "Keyring Error.\nThis error may caused by OS' secret manager, the rsa private key cannot be saved or read."
        ),
        context(false)
    )]
    KeyringError { source: keyring::Error },
    #[snafu(
        display("Encryption Error. Cannot encrypt or decrypt.\nIf it's a decrypt error, maybe it's the private key stored in keyring being incorrect, modified or recreated."),
        context(false)
    )]
    EncryptionError { source: rsa::Error },
    #[snafu(display("IO error. Cannot operate the file."), context(false))]
    IoError { source: std::io::Error },
    #[snafu(display("Cannot collect from Source"))]
    CollectFailed,
}

pub type ConfigResult<T> = Result<T, ConfigError>;
