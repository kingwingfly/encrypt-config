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
[![LinkedIn][linkedin-shield]][linkedin-url]



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

However, they usually have limitation on the secret length. For example, `Keychain` only allows 255 bytes for the secret,  `Credential Manager` is even shorter. So we can't store a long secret in it.

Another solution is to store the secret in a file and encrypt it with a rsa public key, and store the private key in the OS' secret manager. This is what this crate does.

In other cases, maybe our secret is not a `String`, but a config `struct`. We can also use this crate to manage it. When invoke [`Config::get`], it will deserialize the config from the cache and return it.

This crate provides 3 ways to manage your config:
- [`Source`]: A normal source, not persisted or encrypted
- [`PersistSource`]: A source that will be persisted to local file, not encrypted
- [`SecretSource`]: A source that will be persisted to local file and encrypted

This crate also has some optional features:
- `persist`: If enabled, you can use the [`PersistSource`] trait.
- `secret`: If enabled, you can use the [`PersistSource`] and the [`SecretSource`] trait.
- `mock`: If enabled, you can use the mock for testing, which will not use the OS' secret manager and automatically delete the config file persisted to disk after the test.
- `derive`: If enabled, you can use the derive macros to implement the [`Source`], [`PersistSource`] and [`SecretSource`] trait.
- `default_config_dir`: If enabled, the default config dir will be used. Implemented through [dirs-next](https://crates.io/crates/dirs-next).
- `protobuf`: If enabled, protobuf will be used instead of json for better performance. (Not implemented yet)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* Rust
* Keyring

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage
_(You may see many `#[cfg(feature = "...")]` in the example below, if you are not familar to Rust, you may not know this attribute is for `Conditinal Compile`, so that I can test it in `cargo test --all-features` automatically to ensure all go right.)_

You can implement the [`Source`], [`PersistSource`] and [`SecretSource`] yourself.
```rust no_run
# #[cfg(feature = "secret")]
# {
use encrypt_config::{Config, SecretSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Foo(String);

struct SecretSourceImpl;

// impl `SecectSource` trait for `SecretSourceImpl`
impl SecretSource for SecretSourceImpl {
	type Value = Foo;
	type Map = Vec<(String, Self::Value)>;

	#[cfg(not(feature = "default_config_dir"))]
	fn path(&self) -> std::path::PathBuf {
		std::path::PathBuf::from("../tests").join("secret.conf")
	}

	#[cfg(feature = "default_config_dir")]
	fn source_name(&self) -> String {
		"secret.conf".to_owned()
	}

	fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
		Ok(vec![("secret".to_owned(), Foo("secret".to_owned()))])
	}
}

let mut config = Config::new("test"); // Now it's empty
let expect = Foo("secret".to_owned());
config.add_secret_source(SecretSourceImpl).unwrap();
assert_eq!(config.get::<_, Foo>("secret").unwrap(), expect);

// upgrade the secret
let new_expect = Foo("new secret".to_owned());
config.upgrade("secret", &new_expect).unwrap();
assert_eq!(config.get::<_, Foo>("secret").unwrap(), new_expect);

// read from disk
let mut config_new = Config::new("test");
config_new.add_secret_source(SecretSourceImpl).unwrap(); // Read secret config from disk
assert_eq!(config_new.get::<_, Foo>("secret").unwrap(), new_expect); // The persist source is brought back
# }
```

You can also use the derive macros.

```rust no_run
# #[cfg(all(feature = "derive", feature = "secret"))]
# {
use encrypt_config::{PersistSource, SecretSource, Source};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Foo(String);

// To derive [`Source`]
#[derive(Source)]
#[source(value(Foo), default([("key".to_owned(), Foo("value".to_owned()))]))]
struct SourceFoo;

//To derive [`PersistSource`]
#[cfg(not(feature = "default_config_dir"))]
#[derive(PersistSource)]
#[source(value(Foo), path("tests/persist.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
struct PersistSourceFoo;

// To derive [`SecretSource`]
#[cfg(not(feature = "default_config_dir"))]
#[derive(SecretSource)]
#[source(value(Foo), path("tests/secret.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
struct SecretSourceFoo;
# }
```

_For more examples, please refer to the [Example](https://github.com/kingwingfly/encrypt-config/blob/dev/examples) or [Documentation](https://docs.rs/encrypt_config)_

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
[linkedin-url]: https://linkedin.com/in/linkedin_username
[product-screenshot]: images/screenshot.png
