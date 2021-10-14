use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};
use headless_chrome::{protocol::page::ScreenshotFormat, Browser, LaunchOptionsBuilder, Tab};
use reqwest::get;
use std::{
    env,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
    time::Duration,
};

use futures::{stream, StreamExt};
use tokio::{fs, time::timeout};

const PARALLEL_REQUESTS: usize = 8;

async fn take_screenshots(
    tab: Arc<Tab>,
    url: url::Url,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = url.as_str();
    if let Ok(Ok(res)) = timeout(Duration::from_secs(10), get(url)).await {
        tab.navigate_to(url)?.wait_until_navigated()?;

        println!(
            "\x1b[0;32m[+] status=\x1b[0m{}\x1b[0;32m title=\x1b[0m{}\x1b[0;32m URL=\x1b[0m{}",
            res.status(),
            tab.get_title()?,
            url
        );
        let filename = url.replace("://", "-").replace("/", "_") + ".png";
        let png_data = tab.capture_screenshot(ScreenshotFormat::PNG, None, true)?;
        fs::write(filename, png_data).await?;
    } else {
        println!("\x1b[0;31m[-] Timed out URL=\x1b[0m {}", url);
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = clap_app!((crate_name!()) =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg URL: +takes_value +required "Website URLs / Filename of file containing URLs")
        (@arg OUTDIR: -o --output +takes_value  "Output directory to save screenshots")
    )
    .get_matches();

    let outdir = matches.value_of("OUTDIR").unwrap_or("screenshots");


    if let Some(url) = matches.value_of("URL") {
        let browser = Browser::new(
            LaunchOptionsBuilder::default()
                .window_size(Some((1280, 720)))
                .build()?,
        )?;

        if fs::metadata(outdir).await.is_err() {
        fs::create_dir(outdir).await?;
    }

        if fs::metadata(url).await.is_ok() {
            let file = std::fs::File::open(url)?;
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
                .map(|url| {
                    tokio::spawn(take_screenshots(
                        browser.new_tab().expect("Failed to create a tab"),
                        url,
                    ))
                })
                .buffer_unordered(PARALLEL_REQUESTS)
                .collect::<Vec<_>>()
                .await;
        } else if let Ok(valid_url) = url::Url::parse(url) {
            let outdir = Path::new(outdir);
            assert!(env::set_current_dir(&outdir).is_ok());
            take_screenshots(browser.new_tab()?, valid_url).await?;
        } else {
            println!("{:?}", fs::metadata(url).await);
        }
    }

    println!("\x1b[0;34m[*] Done :D\x1b[0m");
    Ok(())
}
