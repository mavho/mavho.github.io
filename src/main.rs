use axum::{http::StatusCode, response::IntoResponse, routing, Router};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::{collections::HashMap, env, fs, io, net::SocketAddr, path::Path, process::exit, thread, time::Duration};
use tower_http::services::{ServeDir,ServeFile};
use chrono::{DateTime,Utc};
use regex::Regex;

mod templates;
const CONTENT_DIR: &str = "content";
const PUBLIC_DIR: &str = "public";
const STATIC_DIR : &str = "static";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let command = Command::new("MaverickWrites")
        .arg(
            Arg::new("generate")
                .short('g')
                .long("generate")
                .action(ArgAction::SetTrue)
        );
    
    let matches:ArgMatches = command.get_matches();

    let generate:bool = matches.get_flag("generate");


    if generate{
        rebuild_site(CONTENT_DIR, PUBLIC_DIR).expect("Rebuilding site");
        println!("Successfully built site.");
        exit(0);
    }


    rebuild_site(CONTENT_DIR, PUBLIC_DIR).expect("Rebuilding site");
    tokio::task::spawn_blocking(move || {
        println!("listening for changes: {}", CONTENT_DIR);
        let mut hotwatch = hotwatch::Hotwatch::new().expect("hotwatch failed to initialize!");
        hotwatch
            .watch(CONTENT_DIR, |_| {
                println!("Rebuilding site");
                rebuild_site(CONTENT_DIR, PUBLIC_DIR).expect("Rebuilding site");
            })
            .expect("failed to watch content folder!");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });

    // static directory route
    //let static_route = Router::new()
    let app = Router::new().nest(
        "/blog",
        routing::get_service(ServeDir::new(PUBLIC_DIR)).handle_error(handle_error),
    ).nest(
        "/static",
        routing::get_service(ServeDir::new(STATIC_DIR)).handle_error(handle_error),
    ).nest(
        "/about",
        routing::get_service(ServeFile::new("static/aboutme.html")).handle_error(handle_error)
    ).fallback(
        routing::get_service(ServeDir::new(PUBLIC_DIR)).handle_error(handle_error)
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("serving site on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/**
 * Extracts out the metadata header (YAML) format from the .md string
 * returns a hashmap of it or none, and removes it from the original markdown string
 */
fn extract_metadata_and_remove_front_matter(markdown: &str) -> (Option<HashMap<String, String>>, String) {

    // trim out the YAML metadata section
    let re = Regex::new(r"(?s)^---\s*(.*?)\s*---").unwrap();

    if let Some(captures) = re.captures(markdown) {
        // metadata section
        let metadata_section = captures.get(1).unwrap().as_str();
        
        // Extract the metadata into a map
        let mut metadata = HashMap::new();
        for line in metadata_section.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                metadata.insert(key, value);
            }
        }

        // Remove the front matter from the markdown string
        let clean_markdown = re.replace(markdown, "").to_string();

        (Some(metadata), clean_markdown)
    } else {
        // No front matter, just return the original markdown
        (None, markdown.to_string())
    }
}


fn rebuild_site(content_dir: &str, output_dir: &str) -> Result<(), anyhow::Error> {
    let _ = fs::remove_dir_all(output_dir);

    let markdown_files: Vec<String> = walkdir::WalkDir::new(content_dir)
        // sort by modified time
        .sort_by(|a,b|
            b.metadata().expect("Unable to parse metadata").created().expect("Unable to parse metadta")
            .cmp(
                &a.metadata().expect("Unable to parse metadata").created().expect("unable to parse metadata")
            )
        )
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            // don't regenerate the about me 
            e.path().display().to_string().ends_with(".md") && !e.path().display().to_string().contains("aboutme.md")
        })
        .map(|e| e.path().display().to_string())
        .collect();
    let mut html_files = Vec::with_capacity(markdown_files.len());
    let mut date_strings = Vec::with_capacity(markdown_files.len());
    let mut md_metadatas:Vec<Option<HashMap<String,String>>> = Vec::with_capacity(markdown_files.len());

    for file in &markdown_files {
        let mut html = templates::HEADER.to_owned();
        let markdown = fs::read_to_string(&file)?;

        let (md_metadata, clean_markdown) = extract_metadata_and_remove_front_matter(&markdown); 

        // If metadata exists, print it (for demonstration purposes)
        // if let Some(ref _metadata) = md_metadata {
        //     println!("Extracted Metadata: {:?}", md_metadata);
        // }

        let parser = pulldown_cmark::Parser::new_ext(&clean_markdown, pulldown_cmark::Options::all());

        // grab file metadata
        let file_metadata = fs::metadata(&file)?;
        let modified_time = file_metadata.modified()?;
        let datetime:DateTime<Utc>= modified_time.into();

        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, parser);

        html.push_str(templates::render_body(&body).as_str());
        html.push_str(templates::FOOTER);

        // writes each .md file into a new HTML file
        let html_file = file
            .replace(content_dir, output_dir)
            .replace(".md", ".html");
        let folder = Path::new(&html_file).parent().unwrap();
        let _ = fs::create_dir_all(folder);
        fs::write(&html_file, html)?;

        html_files.push(html_file);
        date_strings.push(datetime);
        md_metadatas.push(md_metadata);
    }

    write_index(html_files,date_strings,md_metadatas, output_dir)?;
    Ok(())
}

fn write_index(
    files: Vec<String>,
    date_strings: Vec<DateTime<Utc>>,
    md_metadatas: Vec<Option<HashMap<String,String>>>,
    output_dir: &str
) -> Result<(), anyhow::Error> {
    let mut html = templates::HEADER.to_owned();
    let body = files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let file = file.trim_start_matches(output_dir).to_string();
            let default_title = file.trim_start_matches("/").trim_end_matches(".html").to_owned();
            let title = md_metadatas[index]
                .as_ref()
                .and_then(|metadata| metadata.get("title"))
                .unwrap_or(&default_title);

            let default_blurb = &"No blurb available".to_string();
            let blurb = md_metadatas[index]
                .as_ref()
                .and_then(|metadata| metadata.get("blurb"))
                .unwrap_or(default_blurb);
        
            let default_date = &date_strings[index].format("%m/%d/%Y").to_string();
            let date_str = md_metadatas[index]
                .as_ref()
                .and_then(|metadata| metadata.get("date"))
                .unwrap_or(default_date);

            format!(
                r#"
                    <div class="entry-overview">
                        <div class="date">{}</div>
                        <div class="detail">
                        <h1><a href="/blog{}">{}</a></h1>
                        <p>{}</p>
                        </div>
                    </div
                "#,
                date_str,
                file,
                title,
                blurb
            )
        })
        .collect::<Vec<String>>()
        .join("<br />\n");

    html.push_str(templates::render_body(&body).as_str());
    html.push_str(templates::FOOTER);

    let index_path = Path::new(&output_dir).join("index.html");
    fs::write(index_path, html)?;
    Ok(())
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}