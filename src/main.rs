use headless_chrome::Browser;
use rustube::blocking::Video;
use rustube::Id;
use seahorse::App;
use seahorse::Context;
use std::env;
use std::error::Error;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("yshorts [URL]")
        .action(default_action);
    app.run(args);
}

fn default_action(c: &Context) {
    if c.args.len() > 0 {
        let video_ids = get_video_ids(&c.args[0]).unwrap();
        for id in video_ids {
            download_video(id);
        }
    } else {
    }
}

fn download_video(id: String) {
    let callback = rustube::Callback::new()
        .connect_on_progress_closure(move |cargs| {
            let curr = cargs.current_chunk.clone() as u64;
            let total = cargs.content_length.unwrap();
            let percent = curr * 100 / total;

            println!("Progress: {}/{} ({}%)", curr, total, percent);
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
    tab.wait_for_element("ytd-continuation-item-renderer")?
        .scroll_into_view()?;
    let content_elem = tab.find_element("#contents")?;

    let links = content_elem.find_elements_by_xpath("//*[@id=\"thumbnail\"]")?;

    for link in links {
        let at_opts = link.get_attributes()?;
        let attributes = at_opts.as_deref().unwrap();
        for attr in attributes.last() {
            if !attr.contains("-1") {
                video_ids.push(attr.replace("/shorts/", ""));
            }
        }
    }

    Ok(video_ids)
}
