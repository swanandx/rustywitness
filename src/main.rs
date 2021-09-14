use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};
use once_cell::sync::{Lazy, OnceCell};
use reqwest::get;
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{fs, process::Command, time::timeout};

static OUTDIR: OnceCell<String> = OnceCell::new();
static CHROME: OnceCell<String> = OnceCell::new();
static MAX_TIME: Lazy<Duration> = Lazy::new(|| Duration::from_secs(10));

async fn take_ss(url: url::Url) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = url.as_str();

    let screenshot = "--screenshot=".to_owned()
        + OUTDIR.get().unwrap()
        + &url.replace("://", "-").replace("/", "_")
        + ".png";
    let mut chrome_command = Command::new(CHROME.get().unwrap())
        .args([
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
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--hide-scrollbars",
            "--incognito",
            "--window-size=1280,720",
            &screenshot,
            url,
        ])
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
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
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-beta",
        "/usr/bin/google-chrome-unstable",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium-browser",
        "/usr/bin/chromium",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
    ];

    let mut found_chrome = String::new();
    for path in chrome_paths {
        if Path::new(path).exists() {
            found_chrome.push_str(path);
            break;
        }
    }

    if found_chrome.is_empty() {
        panic!("Chrome / Chromium not found :(\nPlease install Chrome/Chromium and try again!");
    };

    found_chrome
}

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

    if fs::metadata(outdir).await.is_err() {
        fs::create_dir(outdir).await?;
    }

    OUTDIR.set(outdir.to_owned() + "/")?;

    if let Some(path) = matches.value_of("PATH") {
        CHROME.set(path.to_string())?;
    } else {
        CHROME.set(find_chrome())?;
    }

    if let Some(url) = matches.value_of("URL") {
        if fs::metadata(url).await.is_ok() {
            let mut handels = Vec::new();
            let lines = fs::read_to_string(url).await?;

            for url in lines.lines() {
                if let Ok(valid_url) = url::Url::parse(url) {
                    handels.push(tokio::spawn(take_ss(valid_url)));
                }
            }

            for handel in handels {
                handel.await?.expect(
                    "Something went wrong while waiting for taking screenshot and saving to file",
                );
            }
        } else if let Ok(valid_url) = url::Url::parse(url) {
            take_ss(valid_url).await?;
        }
    }

    println!("\x1b[0;34m[*] Done :D\x1b[0m");
    Ok(())
}
