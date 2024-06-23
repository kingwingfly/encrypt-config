//! The error types of `encrypt config`.

use snafu::Snafu;

/// The Error types of `encrypt config`, which is implemented by [`snafu`].
#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub enum ConfigError {
    /// This error will be returned when the key is not found in the config.
    #[snafu(display("The type `{}` not found in Config", r#type))]
    ConfigNotFound {
        /// The type which is not found in the config.
        r#type: String,
    },
    /// This error will be returned when the value cannot seriliazed or deserialized.
    #[snafu(
        display("Serde Error. Cannot seriliaze or deseriliaze."),
        context(false)
    )]
    SerdeError {
        /// The error returned by `serde_json`.
        source: serde_json::Error,
    },
    #[cfg(feature = "secret")]
    /// This error will be returned when the encrypter cannot be deserialized from keyring password. This may caused by the private key stored in keyring being incorrect, modified or recreated.
    #[snafu(display("Failed to deseriliaze encrypter from keyring."))]
    LoadEncrypterFailed,
    #[cfg(feature = "secret")]
    /// This error will be returned when the OS' secret manager cannot be accessed.
    #[snafu(
        display(
            "Keyring Error.\nThis error may caused by OS' secret manager, the rsa private key cannot be saved or read."
        ),
    )]
    KeyringError,
    /// This error will be returned when the encryption or decryption failed.
    #[snafu(
        display("Encryption Error. Cannot encrypt or decrypt.\nIf it's a decrypt error, maybe it's the private key stored in keyring being incorrect, modified or recreated."),
        context(false)
    )]
    #[cfg(feature = "secret")]
    EncryptionError {
        /// The error returned by `rsa`.
        source: rsa::Error,
    },
    /// This error will be returned when the config cannot be saved to or read from the file.
    #[snafu(display("IO error. Cannot operate the file."), context(false))]
    IoError {
        /// The error returned by `std::io`.
        source: std::io::Error,
    },
}

/// The Result type of `encrypt config`, which is implemented by [`snafu`].
pub type ConfigResult<T> = Result<T, ConfigError>;
