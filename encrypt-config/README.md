<a name="readme-top"></a>

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
<h3 align="center">encrypt-config</h3>

  <p align="center">
    A rust crate to manage, persist and encrypt your configurations.
    <br />
    <a href="https://docs.rs/encrypt_config"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/kingwingfly/encrypt-config">View Demo</a>
    ·
    <a href="https://github.com/kingwingfly/encrypt-config/issues">Report Bug</a>
    ·
    <a href="https://github.com/kingwingfly/encrypt-config/issues">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#changelog">Changelog</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

Sometimes, we need to store config in our application that we don't want to expose to the public. For example, the database password, the api key, etc.

One solution is to store them in the OS' secret manager, such as `Keychain` on macOS, `Credential Manager` on Windows, `libsecret` on Linux.

However, they usually have limitation on the secret length. For example, `Keychain` only allows 255 bytes for the secret, `Credential Manager` is even shorter. So we can't store a long secret in it.

Another solution is to store the secret in a file and encrypt it with a rsa public key, and store the private key in the OS' secret manager. This is what this crate does.

This crate provides 3 ways to manage your config:
- [`NormalSource`]: A normal source, not persisted or encrypted
- [`PersistSource`]: A source that will be persisted to local file, not encrypted
- [`SecretSource`]: A source that will be persisted to local file and encrypted

This crate also has some optional features:
- `persist`: If enabled, you can use the [`PersistSource`] trait.
- `secret`: If enabled, you can use the [`PersistSource`] and the [`SecretSource`] trait.
- `mock`: If enabled, you can use the mock for testing, which will not use the OS' secret manager.
- `default_config_dir`: If enabled, the default config dir will be used. Implemented through [dirs](https://crates.io/crates/dirs).
- `protobuf`: If enabled, protobuf will be used instead of json for better performance. (WIP)

### Causion

One of `linux-secret-service` and `linux-keyutils` features should be enabled on Linux, or a compile error will be raised.


<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* Rust
* Keyring

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage
### Import
```toml
[target.'cfg(target_os = "linux")'.dependencies]
encrypt_config = { version = "0.2", features ["full", "linux-secret-service"] }

[target.'cfg(not(target_os = "linux"))'.dependencies]
encrypt_config = { version = "0.2", features ["full"] }
```
### Example
```rust no_run
# #[cfg(all(feature = "full", feature = "mock", feature = "default_config_dir"))]
# {
use encrypt_config::{Config, NormalSource, PersistSource, SecretSource};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Default, NormalSource)]
struct NormalConfig {
    count: usize,
}

#[derive(Default, Serialize, Deserialize, PersistSource)]
#[source(name = "persist_config.json")]
struct PersistConfig {
    name: String,
    age: usize,
}

#[derive(Default, Serialize, Deserialize, SecretSource)]
#[source(name = "secret_config", keyring_entry = "secret")]
struct SecretConfig {
    password: String,
}

fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut config = Config::default();
        config.add_normal_source::<NormalConfig>().unwrap();
        config.add_persist_source::<PersistConfig>().unwrap();
        config.add_secret_source::<SecretConfig>().unwrap();
        config
    })
}

let cfg = config();
{
    let normal_config = cfg.get::<NormalConfig>().unwrap();
    assert_eq!(normal_config.count, 0);
}
let mut normal_config = cfg.get_mut::<NormalConfig>().unwrap();
normal_config.count = 42;
assert_eq!(normal_config.count, 42);

let jh = std::thread::spawn(|| {
    // work in another thread
    let cfg = config();
    let mut persist_config = cfg.get_mut::<PersistConfig>().unwrap();
    persist_config.name = "Louis".to_string();
    persist_config.age = 22;
    // save to file
    persist_config.save().unwrap();
});
let cfg = config();
let mut secret_config = cfg.get_mut::<SecretConfig>().unwrap();
secret_config.password = "123456".to_string();
// encrypt and save to file
secret_config.save().unwrap();
jh.join().unwrap();

// let's new a config in the next start
let mut config = Config::default();
config.add_normal_source::<NormalConfig>().unwrap();
config.add_persist_source::<PersistConfig>().unwrap();
config.add_secret_source::<SecretConfig>().unwrap();

// normal config will not be saved
assert_eq!(config.get::<NormalConfig>().unwrap().count, 0);
// persist config will be saved
assert_eq!(config.get::<PersistConfig>().unwrap().name, "Louis");
// secret config will be encrypted
assert_eq!(config.get::<SecretConfig>().unwrap().password, "123456");

// The secret config file should not be able to load directly
let encrypted_file = std::fs::File::open(SecretConfig::path()).unwrap();
assert!(serde_json::from_reader::<_, SecretConfig>(encrypted_file).is_err());
# }
```

_For more examples, please refer to the [Example](https://github.com/kingwingfly/encrypt-config/tree/dev/tests) or [Documentation](https://docs.rs/encrypt_config)_

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- CHANGELOG -->
## Changelog

- v0.1.x -> v0.2.x: A broken change has been made. Heavily refactored with `std::any` and methods from `dependencies injection`.

[more detailed changelog](https://github.com/kingwingfly/encrypt-config/blob/dev/CHANGELOG.md)

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- ROADMAP -->
## Roadmap

- [ ] Enable protobuf instead of json for better performance

See the [open issues](https://github.com/kingwingfly/encrypt-config/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Louis - 836250617@qq.com

Project Link: [https://github.com/kingwingfly/encrypt-config](https://github.com/kingwingfly/encrypt-config)

<p align="right">(<a href="#readme-top">back to top</a>)</p>




<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/kingwingfly/encrypt-config.svg?style=for-the-badge
[contributors-url]: https://github.com/kingwingfly/encrypt-config/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/kingwingfly/encrypt-config.svg?style=for-the-badge
[forks-url]: https://github.com/kingwingfly/encrypt-config/network/members
[stars-shield]: https://img.shields.io/github/stars/kingwingfly/encrypt-config.svg?style=for-the-badge
[stars-url]: https://github.com/kingwingfly/encrypt-config/stargazers
[issues-shield]: https://img.shields.io/github/issues/kingwingfly/encrypt-config.svg?style=for-the-badge
[issues-url]: https://github.com/kingwingfly/encrypt-config/issues
[license-shield]: https://img.shields.io/github/license/kingwingfly/encrypt-config.svg?style=for-the-badge
[license-url]: https://github.com/kingwingfly/encrypt-config/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[product-screenshot]: images/screenshot.png
