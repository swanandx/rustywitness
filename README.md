<h1 align="center">
    rustywitness 
</h1>

<div align="center">
  🦀 🌐 📸
</div>
<div align="center">
  <strong>Web screenshot utility</strong>
</div>
<div align="center">
  A CLI tool for getting screenshots of URLs using headless chrome
</div>

<br />

<div align="center">
  <!-- Twitter -->
  <a href="https://twitter.com/_swanandx">
    <img src="https://img.shields.io/badge/twitter-%40__swanandx-blue"
      alt="@_swanandx" />
  </a>
  <!-- GitHub issues -->
  <a href="https://github.com/swanandx/rustywitness/issues">
    <img src="https://img.shields.io/github/issues/swanandx/rustywitness"
      alt="GitHub issues" />
  </a>
  <!-- GitHub stars -->
  <a href="https://github.com/swanandx/rustywitness/stargazers">
    <img src="https://img.shields.io/github/stars/swanandx/rustywitness"
      alt="GitHub stars" />
  </a>
  <!-- GitHub forks -->
  <a href="https://github.com/swanandx/rustywitness/network">
    <img src="https://img.shields.io/github/forks/swanandx/rustywitness"
      alt="GitHub forks" />
  </a>
  <!-- GitHub license -->
  <a href="https://github.com/swanandx/rustywitness/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/swanandx/rustywitness"
      alt="GitHub license" />
  </a>
</div>

<div align="center">
  <sub>Built with ❤︎ by
  <a href="https://twitter.com/_swanandx">swanandx</a> and
  <a href="https://github.com/swanandx/rustywitness/graphs/contributors">
    contributors
  </a>
</div>
<!-- Thnx to choo for above README design <3 https://github.com/choojs/choo/blob/master/README.md -->
    
    
# 📝 Summary
`rustywitness` is a website screenshot utility written in Rust, that uses Chrome Headless to generate screenshots of website interfaces using the CLI.


# 🔭 Installation
### Download executable 📈

 You can directly download executable and run it. No need for any installation.
 - Check releases [here](https://github.com/swanandx/rustywitness/releases/).


### Using `cargo` 🦀

- `cargo install rustywitness`


### Build it from source 🎯

Clone repository

- `git clone https://github.com/swanandx/rustywitness && cd rustywitness`

then build and run
- `cargo run`
e.g. `cargo run -- <URL/FILENAME> [OPTIONS]`

OR

- `cargo build --release`
- `cd target/release/`
- `./rustywitness`
e.g. `./rustywitness <URL/FILENAME> [OPTIONS]`
    
    
    
# 🧰 Usage
```
rustywitness 0.1.0
swanandx
A CLI tool for getting screenshots of URLs using headless chrome

USAGE:
    rustywitness [OPTIONS] <URL>

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --height <HEIGHT>    Height of screenshot (default 900)
    -m, --max <MAX>          Maximum number of parallel tabs (default 4)
    -o, --output <OUTDIR>    Output directory to save screenshots (default 'screenshots')
    -w, --width <WIDTH>      Width of screenshot (default 1400)

ARGS:
    <URL>    Website URL / Filename of file containing URLs 
```

- screenshot a single website `rustywitness https://example.com`
- screenshot domain-list `rustywitness lives.txt`



You don't need to worry about remembering flags, it can determine if the argument is a file or a single URL!


# 🚧 Contributing

There is always scope for improvements and bugs to be fixed! Just open a issue or submit a PR.

# Acknowledgement

[Similar](https://github.com/sensepost/gowitness) [projects](https://github.com/michenriksen/aquatone) were very helpful as a reference. Special thanks to [siddicky](https://github.com/siddicky) and [Drago](https://github.com/vaishnavpardhi) for testing it!
