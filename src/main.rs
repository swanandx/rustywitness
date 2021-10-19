use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};
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
const PARALLEL_REQUESTS: usize = 4;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = clap_app!((crate_name!()) =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg URL: +takes_value +required "Website URLs / Filename of file containing URLs")
        (@arg PATH: -p --path +takes_value  "Specify valid path to Chrome/Chromium")
        (@arg OUTDIR: -o --output +takes_value  "Output directory to save screenshots")
    )
    .get_matches();

    let outdir = matches.value_of("OUTDIR").unwrap_or("screenshots");

    if fs::metadata(outdir).is_err() {
        fs::create_dir(outdir)?;
    }

    if let Some(path) = matches.value_of("PATH") {
        CHROME.set(path.to_string())?;
    } else {
        CHROME.set(find_chrome())?;
    }

    if let Some(url) = matches.value_of("URL") {
        if fs::metadata(url).is_ok() {
            let file = fs::File::open(url)?;
            let lines = BufReader::new(file).lines();

            let mut urls = Vec::new();

            for line in lines.flatten() {
                if let Ok(url) = url::Url::parse(&line) {
                    urls.push(url);
                }
            }

            let outdir = Path::new(outdir);
            assert!(env::set_current_dir(&outdir).is_ok());

            stream::iter(urls)
                .map(|url| tokio::spawn(take_screenshot(url)))
                .buffer_unordered(PARALLEL_REQUESTS)
                .collect::<Vec<_>>()
                .await;
        } else if let Ok(valid_url) = url::Url::parse(url) {
            let outdir = Path::new(outdir);
            assert!(env::set_current_dir(&outdir).is_ok());
            take_screenshot(valid_url).await?;
        }
    }

    println!("\x1b[0;34m[*] Done :D\x1b[0m");
    Ok(())
}

async fn take_screenshot(url: url::Url) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = url.as_str();

    let screenshot =
        "--screenshot=".to_owned() + &url.replace("://", "-").replace("/", "_") + ".png";
    let mut chrome_command = Command::new(CHROME.get().unwrap())
        .args(&ARGS)
        .arg(screenshot)
        .arg(url)
        .stderr(Stdio::piped())
        .spawn()?;

    if let Ok(Ok(res)) = timeout(*MAX_TIME, get(url)).await {
        println!(
            "\x1b[0;32m[+] Status:\x1b[0m {} \x1b[0;32m => URL: \x1b[0m {}",
            res.status(),
            url
        );
        chrome_command.wait().await?;
    } else {
        println!("\x1b[0;31m[-] Timed out URL:\x1b[0m {}", url);
        chrome_command.kill().await?;
    }

    Ok(())
}

fn find_chrome() -> String {
    let chrome_paths = [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome",
        "chrome-browser",
        "chrome.exe",
    ];

    let mut found_chrome = String::new();
    for path in chrome_paths {
        if which::which(path).is_ok() {
            found_chrome.push_str(path);
            break;
        }
    }

    if found_chrome.is_empty() {
        panic!(
            "Chrome / Chromium not found :(\nPlease install Chrome/Chromium or specify path to it!"
        );
    };

    found_chrome
}

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
