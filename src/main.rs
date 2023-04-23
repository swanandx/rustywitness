use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use reqwest::get;
use std::{
    env,
    io::{BufRead, BufReader},
    path::Path,
    time::Duration,
};
use tokio::{fs, time::timeout};

use futures::StreamExt;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};
use chromiumoxide::handler::viewport::Viewport;
use chromiumoxide::Page;

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
            Arg::with_name("OUTDIR")
                .help("Output directory to save screenshots (default 'screenshots')")
                .takes_value(true)
                .short("o")
                .long("output"),
        )
        .arg(
            Arg::with_name("MAX")
                .help("Maximum number of parallel tabs (default 4)")
                .takes_value(true)
                .short("m")
                .long("max"),
        )
        .arg(
            Arg::with_name("WIDTH")
                .help("Width of screenshot (default 1400)")
                .takes_value(true)
                .short("w")
                .long("width"),
        )
        .arg(
            Arg::with_name("HEIGHT")
                .help("Height of screenshot (default 900)")
                .takes_value(true)
                .short("h")
                .long("height"),
        )
        .arg(
            Arg::with_name("PATH")
                .help("Path to chrome executable")
                .takes_value(true)
                .short("p")
                .long("path"),
        )
        .get_matches();

    let outdir = matches.value_of("OUTDIR").unwrap_or("screenshots");
    let width: u32 = matches
        .value_of("WIDTH")
        .unwrap_or("1400")
        .parse()
        .unwrap_or(1400);
    let height: u32 = matches
        .value_of("HEIGHT")
        .unwrap_or("900")
        .parse()
        .unwrap_or(900);

    let mut config = BrowserConfig::builder()
        .no_sandbox()
        .window_size(width, height)
        .viewport(Viewport {
            width,
            height,
            device_scale_factor: None,
            emulating_mobile: false,
            is_landscape: false,
            has_touch: false,
        });

    if let Some(path) = matches.value_of("PATH") {
        config = config.chrome_executable(path);
    }

    let (browser, mut handler) = Browser::launch(config.build()?).await?;

    let _handle = tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let mut parallel_tabs: usize = 4;
    if let Some(max) = matches.value_of("MAX") {
        parallel_tabs = max.parse()?;
    }

    if fs::metadata(outdir).await.is_err() {
        fs::create_dir(outdir).await?;
    }

    if let Some(url) = matches.value_of("URL") {
        if fs::metadata(url).await.is_ok() {
            let file = std::fs::File::open(url)?;
            let lines = BufReader::new(file).lines();

            let mut urls = vec![Vec::new(); parallel_tabs];
            let mut pt = 0;

            // Only take valid URLs
            // push them in urls in round robin manner
            for line in lines.flatten() {
                if let Ok(url) = url::Url::parse(&line) {
                    urls[pt].push(url);
                    pt += 1;
                    pt %= parallel_tabs;
                }
            }

            // Set current working directory to output directory
            // So that we can save screenshots in it without specifying whole path.
            env::set_current_dir(Path::new(outdir))?;

            let mut handles = Vec::new();

            for chunk in urls {
                let n_tab = browser.new_page("about:blank").await?;
                let h = tokio::spawn(take_screenshots(n_tab, chunk));
                handles.push(h);
            }

            for handle in handles {
                handle.await?.expect(
                    "Something went wrong while waiting for taking screenshot and saving to file",
                );
            }
        } else if let Ok(valid_url) = url::Url::parse(url) {
            env::set_current_dir(Path::new(outdir))?;
            let n_tab = browser.new_page("about:blank").await?;
            take_screenshots(n_tab, vec![valid_url]).await?;
        } else {
            eprintln!(
                "\x1b[0;31mInvalid URL:\x1b[0m {} {:?}",
                url,
                url::Url::parse(url)
            );
        }
    }

    println!("\x1b[0;34m[*] Done :D\x1b[0m");

    Ok(())
}

async fn take_screenshots(
    page: Page,
    urls: Vec<reqwest::Url>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for url in urls {
        let url = url.as_str();
        if let Ok(Ok(res)) = timeout(Duration::from_secs(10), get(url)).await {
            let filename = url.replace("://", "-").replace('/', "_") + ".png";
            page.goto(url)
                .await?
                .save_screenshot(
                    CaptureScreenshotParams::builder()
                        .format(CaptureScreenshotFormat::Png)
                        .build(),
                    filename,
                )
                .await?;
            println!(
                "\x1b[0;32m[+] status=\x1b[0m{}\x1b[0;32m title=\x1b[0m{}\x1b[0;32m URL=\x1b[0m{}",
                res.status(),
                page.get_title().await?.unwrap_or_default(),
                url
            );
        } else {
            println!("\x1b[0;31m[-] Timed out URL=\x1b[0m {}", url);
        }
    }

    Ok(())
}
