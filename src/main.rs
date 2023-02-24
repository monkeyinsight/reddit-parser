use std::io::Cursor;
use reqwest;
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::env;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct Source {
    url: String,
    width: u32,
    height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Preview {
    source: Source,
}

#[derive(Serialize, Deserialize, Debug)]
struct Previews<T> {
    images: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PostData {
    title: String,
    preview: Option<Previews<Preview>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    data: PostData,
}

#[derive(Serialize, Deserialize, Debug)]
struct Posts<T> {
    children: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
    data: Posts<Post>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn fetch_url(url: String, filename: String) -> Result<()> {
    let response = reqwest::get(&url).await?;
    fs::create_dir_all("./images/")?;
    let mut file = fs::File::create(format!("./images/{}", filename))?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

async fn fetch_posts(posts: Vec<&Post>) {
    for post in posts {
        println!("{}", post.data.title);

        match &post.data.preview {
            Some(x) => {
                let url = x.images.first().unwrap().source.url.replace("&amp;", "&");

                let re = Regex::new(r"/([^/]+?)\?").unwrap();
                let filename = re.find(&url).unwrap().as_str().replace("?", "").replace("/", "");
                println!("{}", filename);

                fetch_url(url.to_string(), filename).await;
            },
            None => println!("No image for post."),
        }

        println!("---------");
    }
}

// tokio let's us use "async" on our main function
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let sub = if &args.len() > &1 {
        &args[1]
    } else {
        "all"
    }.to_string();

    let client = reqwest::Client::new();
    println!("{}", format!("https://www.reddit.com/r/{}/hot.json", sub));
    let response = client.get(format!("https://www.reddit.com/r/{}/hot.json", sub))
        .send()
        .await
        .unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<APIResponse>().await {
                // Ok(parsed) => println!("Success! {:?}", parsed),
                Ok(parsed) => fetch_posts(parsed.data.children.iter().collect()).await,
                Err(_) => println!("Hm, the response didn't match the shape we expected."),
            }
        }
        other => {
            panic!("Uh oh! Something unexpected happened: {:?}", other);
        }
    };
}