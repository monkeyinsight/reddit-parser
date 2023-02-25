// use std::io::Cursor;
use clap::Parser;
use reqwest::{Client};
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::fs;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   #[arg(short, long)]
   u: String,
   #[arg(short, long)]
   p: String,
   #[arg(short, long)]
   room: String,
   #[arg(short, long, default_value="all")]
   sub: String,
   #[arg(short, long, default_value="hot")]
   t: String,
}


#[derive(Serialize, Deserialize, Debug)]
struct Source {
    url: String,
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

async fn fetch_url(url: String, title: &String) -> Result<()> {
    // let response = reqwest::get(&url).await?;
    // fs::create_dir_all("./images/")?;

    // let mut file = fs::File::create(format!("./images/{}", filename))?;
    // let mut content = Cursor::new(response.bytes().await?);
    // std::io::copy(&mut content, &mut file)?;

    println!("{}", title);

    upload(url, title).await;

    Ok(())
}

async fn upload(url: String, title: &String) {
    let args = Args::parse();

    let mut params = HashMap::new();
    params.insert("email", args.u);
    params.insert("password", args.p);

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let response = client.post("https://multimedia.chat/api/auth/login")
        .form(&params)
        .send()
        .await
        .unwrap();

    let cookies = response.cookies().collect::<Vec<_>>();

    println!("Cookies: {:?}", cookies);

    let text = response.text().await.unwrap();
    println!("{:?}", text);

    let mut upload = HashMap::new();
    upload.insert("title", title);
    upload.insert("url", &url);

    let response = client.post(format!("https://multimedia.chat/api/channels/{}/medias/url", args.room))
        .form(&upload)
        .send()
        .await
        .unwrap();

    let text = response.text().await.unwrap();
    println!("{:?}", text);
}

async fn fetch_posts(posts: Vec<&Post>) {
    for post in posts {
        let title = &post.data.title;

        match &post.data.preview {
            Some(x) => {
                let url = x.images.first().unwrap().source.url.replace("&amp;", "&");

                let re = Regex::new(r"/([^/]+?)\?").unwrap();
                let filename = re.find(&url).unwrap().as_str().replace("?", "").replace("/", "");

                match fs::metadata(format!("./images/{}", filename)).is_ok() {
                    false => {
                        println!("{}", filename);

                        if let Err(_e) = fetch_url(url.to_string(), title).await {
                            println!("Error fetching image.");
                        }

                        return;
                    },
                    true => {
                        println!("File already exist.");
                    }
                }
            },
            None => println!("No image."),
        }

        println!("-------");
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let client = Client::new();
    println!("Reading /r/{}/{}", args.sub, args.t);
    let response = client.get(format!("https://www.reddit.com/r/{}/{}.json", args.sub, args.t))
        .send()
        .await
        .unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<APIResponse>().await {
                Ok(parsed) => fetch_posts(parsed.data.children.iter().collect()).await,
                Err(_) => println!("Hm, the response didn't match the shape we expected."),
            }
        }
        other => {
            panic!("Uh oh! Something unexpected happened: {:?}", other);
        }
    };
}