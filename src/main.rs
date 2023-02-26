use clap::Parser;
use headless_chrome::Browser;
use rustube::blocking::Video;
use rustube::Id;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// youtube shorts url
    #[arg(short, long)]
    url: String,
}

fn main() {
    let args = Args::parse();

    let video_ids = get_video_ids(&args.url).unwrap();

    for id in video_ids {
        download_video(id);
    }
}

fn download_video(id: String) {
    let callback = rustube::Callback::new()
        .connect_on_progress_closure(move |cargs| {
            let curr = cargs.current_chunk.clone() as u64;
            let total = cargs.content_length.unwrap();
            let percent = curr * 100 / total;

            println!("\rProgress: {}/{} ({}%)", curr, total, percent);
        })
        .connect_on_complete_closure(move |_| {
            println!("Download complete");
        });

    let id = Id::from_string(id).unwrap();
    let video = Video::from_id(id.into_owned()).unwrap();
    let title = &video.video_details().title;
    println!("{title}");
    video
        .best_quality()
        .unwrap()
        .blocking_download_with_callback(callback)
        .unwrap();
}

fn get_video_ids(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut video_ids: Vec<String> = vec![];
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;
    let ytd_continuation = tab.wait_for_element("ytd-continuation-item-renderer");

    if ytd_continuation.is_ok() {
        ytd_continuation?.scroll_into_view()?;
    }

    let content_elem = tab.find_element("#contents")?;

    let links = content_elem.find_elements_by_xpath("//*[@id=\"thumbnail\"]")?;

    for link in links {
        let at_opts = link.get_attributes()?;
        let attributes = at_opts.as_deref().unwrap();
        for attr in attributes.last() {
            if !attr.contains("-1") {
                println!("Get video id: {attr}");
                video_ids.push(attr.replace("/shorts/", ""));
            }
        }
    }

    Ok(video_ids)
}
