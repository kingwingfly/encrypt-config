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
  <a href="https://github.com/kingwingfly/encrypt-config">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">encrypt-config</h3>

  <p align="center">
    A rust crate to manage, persist and encrypt your configurations.
    <br />
    <a href="https://github.com/kingwingfly/encrypt-config"><strong>Explore the docs »</strong></a>
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
    <li><a href="#getting-started">Getting Started</a></li>
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

[![Product Name Screen Shot][product-screenshot]](https://github.com/kingwingfly/encrypt-config)

A rust crate to manage, persist, encrypt configurations.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* Rust
* Keyring

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

Details here: [Example](example/eamples.rs)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage
```rust
use encrypt_config::{Config, ConfigKey, ConfigResult, SecretSource};

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct Bar(String);

struct SecretSourceImpl;

// impl `SecectSource` trait for `SecretSourceImpl`
impl SecretSource for SecretSourceImpl {
    type Value = Bar;

    // The key to query from `Config`
    fn source_name(&self) -> ConfigKey {
        "secret_test".to_owned()
    }

    // The default value if the persist file has not been created
    fn default(&self) -> Self::Value {
        Bar("world".to_owned())
    }

    // Where the encypted file is in. Don't need if turning on `default_config_dir` feature.
    fn path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("tests").join(self.source_name())
    }
}

// `test` is the name of rsa private key in OS' secret manager
let mut config = Config::new("test");
config.add_secret_source(SecretSourceImpl).unwrap();

// `get` will do a deserialization
let v: Bar = config.get("secret_test").unwrap();
assert_eq!(v, Bar("world".to_owned()));

// `upgrade` will return a `Patch`
let patch = SecretSourceImpl.upgrade(&Bar("Louis".to_owned())).unwrap();
// No change will happen until the `Patch` is applied
patch.apply(&mut config).unwrap();
let v: Bar = config.get("secret_test").unwrap();
assert_eq!(v, Bar("Louis".to_owned()));
```

_For more examples, please refer to the [Documentation](https://docs.rs/encrypt-config/latest/encrypt-config)_

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



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* None
* []()

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
