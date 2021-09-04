use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};
#[allow(unused_doc_comments)]
use headless_chrome::{
    protocol::page::ScreenshotFormat, protocol::target::methods::CreateTarget, Browser,
};
use std::fs;


// TODOs
// This is just a MVP [ minimum viable product]
// We need to take height and width from user too (optional)
// path to save file can also be specified. default being "_ss/"
// allow use of multiple urls. we can just use navigate function to navigate.
fn main() -> Result<(), failure::Error> {
    let browser = Browser::default()?;

    let matches = clap_app!((crate_name!()) =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg URL: +takes_value +required "website url")
    )
    .get_matches();

    if let Some(url) = matches.value_of("URL") {
        let tab = browser.new_tab_with_options(CreateTarget {
            url: url,
            width: Some(1920),
            height: Some(1080),
            browser_context_id: None,
            enable_begin_frame_control: None,
        })?;

        let png_data =
            tab.wait_until_navigated()?
                .capture_screenshot(ScreenshotFormat::PNG, None, true)?;

        fs::write("ss.png", png_data).expect("Failed to write data");
    }

    Ok(())
}
