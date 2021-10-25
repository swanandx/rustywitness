use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use colored::Colorize;
use futures::{stream, StreamExt};
use once_cell::sync::{Lazy, OnceCell};
use reqwest::get;
use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::Path,
    process::Stdio,
    time::Duration,
};
use tokio::{process::Command, time::timeout};

static CHROME: OnceCell<String> = OnceCell::new();
static MAX_TIME: Lazy<Duration> = Lazy::new(|| Duration::from_secs(10));

#[cfg(windows)]
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("URL")
                .help("Website URL / Filename of file containing URLs")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("PATH")
                .help("Specify valid path to Chrome/Chromium")
                .takes_value(true)
                .short("p")
                .long("path"),
        )
        .arg(
            Arg::with_name("OUTDIR")
                .help("Output directory to save screenshots (default 'screenshots')")
                .takes_value(true)
                .short("o")
                .long("output"),
        )
        .arg(
            Arg::with_name("MAX")
                .help("Maximum number of parallel tasks (default 4)")
                .takes_value(true)
                .short("m")
                .long("max"),
        )
        .get_matches();

    let outdir = matches.value_of("OUTDIR").unwrap_or("screenshots");

    // Use user specified path for chrome,
    // if not specified, check if chrome is in path.
    CHROME.set({
        if let Some(path) = matches.value_of("PATH") {
            path.to_string()
        } else {
            default_executable()?
        }
    })?;

    let mut parallel_tasks: usize = 4;
    if let Some(max) = matches.value_of("MAX") {
        parallel_tasks = max.parse()?;
    }

    if fs::metadata(outdir).is_err() {
        fs::create_dir(outdir)?;
    }

    if let Some(url) = matches.value_of("URL") {
        if fs::metadata(url).is_ok() {
            let file = fs::File::open(url)?;
            let lines = BufReader::new(file).lines();

            let mut urls = Vec::new();

            //Only take valid URLs
            for line in lines.flatten() {
                if let Ok(url) = url::Url::parse(&line) {
                    urls.push(url);
                }
            }

            // Set current working directory to output directory
            // So that we can save screenshots in it without specifying whole path.
            env::set_current_dir(Path::new(outdir))?;

            // Limit the number of parallel tasks using buffer_unordered()
            stream::iter(urls)
                .map(|url| tokio::spawn(take_screenshot(url)))
                .buffer_unordered(parallel_tasks)
                .collect::<Vec<_>>()
                .await;
        } else if let Ok(valid_url) = url::Url::parse(url) {
            env::set_current_dir(Path::new(outdir))?;
            take_screenshot(valid_url).await?;
        } else {
            eprintln!(
                "{} {} {:?}",
                "Invalid URL =>".red(),
                url,
                url::Url::parse(url)
            );
        }
    }

    println!("{}", "[*] Done :D".cyan());
    Ok(())
}

async fn take_screenshot(url: url::Url) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = url.as_str();

    let screenshot_argument = {
        #[cfg(not(windows))]
        {
            format!(
                "--screenshot={}.png",
                &url.replace("://", "-").replace("/", "_")
            )
        }

        #[cfg(windows)]
        {
            let mut path = env::current_dir()?;
            path.push(url.replace("://", "-").replace("/", "_") + ".png");
            format!("--screenshot={}", path.display())
        }
    };

    let mut chrome_command = Command::new(CHROME.get().unwrap())
        .args(&ARGS)
        .arg(screenshot_argument)
        .arg(url)
        .stderr(Stdio::piped())
        .spawn()?;

    if let Ok(Ok(res)) = timeout(*MAX_TIME, get(url)).await {
        println!(
            "{} {} => {} {}",
            "[+] Status:".green(),
            res.status(),
            "URL:".green(),
            url
        );
        chrome_command.wait().await?;
    } else {
        eprintln!("{} {}", "[-] Timed out URL:".red(), url);
        chrome_command.kill().await?;
    }

    Ok(())
}

// Reference :-
// https://github.com/atroche/rust-headless-chrome/blob/master/src/browser/process.rs
fn default_executable() -> Result<String, &'static str> {
    if let Ok(path) = std::env::var("CHROME") {
        if std::path::Path::new(&path).exists() {
            return Ok(path);
        }
    }

    for app in [
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome",
        "chrome-browser",
    ] {
        if which::which(app).is_ok() {
            return Ok(app.to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let path = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
        if std::path::Path::new(&path).exists() {
            return Ok(path.to_string());
        }
    }

    #[cfg(windows)]
    {
        if let Ok(path) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\chrome.exe")
            .and_then(|key| key.get_value::<String, _>(""))
        {
            return Ok(path);
        }
    }

    Err("Could not auto detect a chrome executable")
}

// Arguments for chrome
static ARGS: [&str; 15] = [
    "--mute-audio",
    "--disable-notifications",
    "--no-first-run",
    "--disable-crash-reporter",
    "--disable-infobars",
    "--disable-sync",
    "--no-default-browser-check",
    "--ignore-certificate-errors",
    "--ignore-urlfetcher-cert-requests",
    "--no-sandbox",
    "--headless",
    "--disable-dev-shm-usage",
    "--hide-scrollbars",
    "--incognito",
    "--window-size=1440,900",
];
