use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};
use fantoccini::ClientBuilder;
use once_cell::sync::Lazy;
use std::env;
use std::process::{Command, Stdio};
use tokio::fs;

struct TemporaryProcess(std::process::Child);

impl Drop for TemporaryProcess {
    fn drop(&mut self) {
        println!("Killing spawned webdriver process=> PID: {}", self.0.id());
        self.0.kill().and_then(|_| self.0.wait()).ok();
    }
}

async fn take_ss(url: url::Url, port: u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = url.as_str();
    let driver_instance = Lazy::new(|| format!("http://localhost:{}", port));
    let caps = Lazy::new(|| {
        let mut caps = serde_json::map::Map::new();
        let chrome_opts = serde_json::json!({ "args": ["--headless"] });
        let firefox_opts = serde_json::json!({ "args": ["--headless"] });
        caps.insert("goog:chromeOptions".to_string(), chrome_opts);
        caps.insert("moz:firefoxOptions".to_string(), firefox_opts);
        caps
    });

    let mut client = ClientBuilder::native()
        .capabilities((*caps).clone())
        .connect(&driver_instance)
        .await?;

    client.set_window_size(1280, 720).await?;

    if let Ok(()) = client.goto(url).await {
        let png_data = client.screenshot().await?;
        client.close().await?;
        let filename =
            "screenshots/".to_owned() + &url.replace("://", "-").replace("/", "_") + ".png";
        fs::write(filename, &png_data).await?;

        println!("\x1b[0;32m[+] Captured screenshot of URL:\x1b[0m {}", url);
    }

    Ok(())
}

async fn run_driver(port: u32, driver_path: Option<&str>) -> TemporaryProcess {
    let key = "PATH";
    let chromedriver = "chromedriver";
    let geckodriver = "geckodriver";
    let mut found_driver: Option<&str> = None;
    if driver_path.is_none() {
        match env::var_os(key) {
            Some(paths) => {
                for mut path in env::split_paths(&paths) {
                    // check if chromedriver exists
                    path.push(chromedriver);
                    if path.as_path().exists() {
                        found_driver = Some(chromedriver);
                        break;
                    } else {
                        path.pop();
                        path.push(geckodriver);
                        if path.as_path().exists() {
                            found_driver = Some(geckodriver);
                            break;
                        }
                    }
                }
            }
            None => println!("{} is not defined in the environment.", key),
        };
    } else {
        found_driver = driver_path;
    }

    let port_arg = format!("--port={}", port);

    let driver_process = if let Some(driver) = found_driver {
        TemporaryProcess(
            Command::new(driver)
                .arg(port_arg)
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Provided driver is not valid"),
        )
    } else {
        panic!("No WebDriver found :(\nThis program need a WebDriver like chromedriver, geckodriver, etc. for execution");
    };

    driver_process
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = clap_app!((crate_name!()) =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg URL: +takes_value +required "Website URLs / Filename of file containing URLs")
        (@arg DRIVER: -d --driver +takes_value  "Specify WebDriver path")
        (@arg PORT: -p --port +takes_value  "Port for running WebDriver")
    )
    .get_matches();

    let port = matches
        .value_of("PORT")
        .unwrap_or("1337")
        .parse()
        .unwrap_or(1337);

    let driver_process = run_driver(port, matches.value_of("DRIVER")).await;

    println!("Started webdriver=> PID: {}", driver_process.0.id());

    if fs::metadata("screenshots/").await.is_err() {
        fs::create_dir("screenshots").await?;
    }

    if let Some(url) = matches.value_of("URL") {
        if fs::metadata(url).await.is_ok() {
            let mut handels = Vec::new();
            let lines = fs::read_to_string(url).await?;

            for url in lines.lines() {
                if let Ok(valid_url) = url::Url::parse(url) {
                    handels.push(tokio::spawn(take_ss(valid_url, port)));
                }
            }

            for handel in handels {
                handel.await?.expect(
                    "Something went wrong while waiting for taking screenshot and saving to file",
                );
            }
        } else if let Ok(valid_url) = url::Url::parse(url) {
            take_ss(valid_url, port).await?;
        }
    }

    println!("\x1b[0;34m[*] Done :D\x1b[0m");
    Ok(())
}
