use atty::{is, Stream};
use aws_sdk_s3::Client;
use clap::Parser;
use futures_util::TryStreamExt;
use kdam::term::Colorizer;
use kdam::{tqdm, BarExt, Column, RichProgress, Spinner};
use regex::Regex;
use remotefs::RemoteFs;
use remotefs_ssh::{ScpFs, SshOpts};
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command(name = "C3P(No O)")]
#[command(bin_name = "c3p")]
#[command(author = "Willem B. <willem.basson@gmail.com>")]
#[command(about = "
All your copies R mine
 🤖　 　　,,''´ ￣ ヽ
　　 　　| |__　 _　|
　 　 　 {{‐'(👁 )Y(👁 )}
  　 　　 !l_l__V^`r'/
　 　　　 ~lr､i_ﾆ_l,'
　　,. r-‐‐]l===l[‐--,r- ､
　 〉､l!　　　｀Y´o　　l!ﾞ,
. //　〉､＿＿L＿＿/il〈.　ﾍ
//　/ }　,'´￣｀ヽ＿{ ﾊV_,ﾍ


  ___  ____  ____
 / __)( __ \\(  _ \\
( (__  (__ ( ) __/
 \\___)(____/(__)")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input to operate on (Use - if you pipe from StdIn)
    input: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Output to operate on (Use - to pipe to StdOut)
    output: Option<String>,
}

#[derive(Debug, PartialEq)]
enum InputKind {
    OrdinaryFile(String),
    StdIn,
    Url(url::Url),
    S3Bucket(String),
    ScpSource(String),
}

#[derive(Debug)]
struct Input {
    kind: InputKind,
}

fn to_input(input: String) -> Input {
    let input_url_result = Url::parse(input.as_str());
    if let Ok(input_url) = input_url_result {
        if input.starts_with("s3://") {
            Input {
                kind: InputKind::S3Bucket(input),
            }
        } else if input.starts_with("scp://") {
            Input {
                kind: InputKind::ScpSource(input),
            }
        } else {
            Input {
                kind: InputKind::Url(input_url),
            }
        }
    } else if input == "-" {
        if is(Stream::Stdin) {
            println!("You said StdIn but you didn't pipe or redirect anything");
        }
        Input {
            kind: InputKind::StdIn,
        }
    } else if input.contains('@') && input.contains(':') {
        Input {
            kind: InputKind::ScpSource(input),
        }
    } else {
        Input {
            kind: InputKind::OrdinaryFile(input),
        }
    }
}

#[derive(Debug, PartialEq)]
enum OutputKind {
    OrdinaryFile(String),
    StdOut,
    Url(url::Url),
    S3Bucket(String),
    ScpTarget(String),
}

#[derive(Debug)]
struct Output {
    kind: OutputKind,
}

fn to_output(output: String) -> Output {
    let output_url_result = Url::parse(output.as_str());
    if let Ok(output_url) = output_url_result {
        if output.starts_with("s3://") {
            Output {
                kind: OutputKind::S3Bucket(output),
            }
        } else if output.starts_with("scp://") {
            Output {
                kind: OutputKind::ScpTarget(output),
            }
        } else {
            Output {
                kind: OutputKind::Url(output_url),
            }
        }
    } else if output == "-" {
        if is(Stream::Stdout) {
            println!("You said StdOut but you didn't pipe or redirect anything");
        }
        Output {
            kind: OutputKind::StdOut,
        }
    } else if output.contains('@') && output.contains(':') {
        Output {
            kind: OutputKind::ScpTarget(output),
        }
    } else {
        Output {
            kind: OutputKind::OrdinaryFile(output),
        }
    }
}

async fn download_from_s3(input: &str, output_path: &str) -> Result<(), String> {
    let shared_config = aws_config::from_env().load().await;
    let client = Client::new(&shared_config);
    let re = Regex::new(r"^s3://(.+?)/(.*)").unwrap();
    let caps = re.captures(input).unwrap();
    let bucket = &caps[1];
    let file = &caps[2];
    println!("bucket {}, file {}", bucket, file);
    let mut object = client
        .get_object()
        .bucket(bucket)
        .key(file.to_string())
        .send()
        .await
        .or(Err(format!("Failed to get s3 file {}/{}", bucket, file)))?;
    let total_size = object.content_length();
    let mut file =
        File::create(output_path).or(Err(format!("Failed to create file '{}'", output_path)))?;
    let mut pb = progress_bar(total_size as usize);
    while let Some(bytes) = object
        .body
        .try_next()
        .await
        .or(Err(format!("Failed to get bytes from stream {}", input)))?
    {
        file.write_all(&bytes)
            .or(Err("Error while writing to file".to_string()))?;
        pb.update(bytes.len());
    }
    Ok(())
}

fn ssh_client(user: &str, pass: &str, host: &str) -> ScpFs {
    let mut client: ScpFs = SshOpts::new(host).username(user).password(pass).into();
    client.connect().unwrap();
    client
}

fn progress_bar(total_size: usize) -> RichProgress {
    RichProgress::new(
        tqdm!(
            total = total_size,
            unit_scale = true,
            unit_divisor = 1024,
            unit = "B"
        ),
        vec![
            Column::Spinner(Spinner::new(&["⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"], 80.0, 1.0)),
            Column::Text("🏎".to_owned()),
            Column::Text("|".to_owned()),
            Column::Percentage(1),
            Column::Text("•".to_owned()),
            Column::CountTotal,
            Column::Text("•".to_owned()),
            Column::Rate,
            Column::Text("•".to_owned()),
            Column::RemainingTime,
        ],
    )
}

async fn download_file(url: &str, output_path: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await.unwrap();
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;
    let mut file =
        File::create(output_path).or(Err(format!("Failed to create file '{}'", output_path)))?;
    let mut stream = res.bytes_stream();
    let mut pb = progress_bar(total_size as usize);
    while let Some(bytes) = stream
        .try_next()
        .await
        .or(Err(format!("Failed to get bytes from stream {}", url)))?
    {
        file.write_all(&bytes)
            .or(Err("Error while writing to file".to_string()))?;
        pb.update(bytes.len());
    }
    pb.write("downloaded".colorize("bold green"));
    Ok(())
}

async fn file_from_std_in(output_path: &str) -> Result<(), String> {
    let mut file =
        File::create(output_path).or(Err(format!("Failed to create file '{}'", output_path)))?;
    let lines_iter = io::stdin().lines().map(|l| l.unwrap());
    for bytes in lines_iter {
        file.write_all(bytes.as_bytes()).unwrap();
        file.write_all(b"\n").unwrap();
    }
    Ok(())
}

async fn file_from_scp(scp_string: String, output_path: &str) -> Result<(), String> {
    let mut file =
        File::create(output_path).or(Err(format!("Failed to create file '{}'", output_path)))?;
    let re = Regex::new(r"^scp://(.+?):(.+?)@(.+?):(.+)").unwrap();
    let caps = re.captures(&scp_string).unwrap();
    let user = &caps[1];
    let pass = &caps[2];
    let host = &caps[3];
    let path = &caps[4];
    println!("user: {}, host: {}, path: {}", user, host, path);
    let mut client = ssh_client(user, pass, host);
    let mut reader = client.open(&Path::new(path)).unwrap();
    let mut buf: Vec<u8> = vec![];
    reader.read_to_end(&mut buf).unwrap();
    file.write_all(&buf)
        .or(Err("Error while writing to file".to_string()))?;
    Ok(())
}

async fn copy(input: Input, output: Output) {
    match input.kind {
        InputKind::OrdinaryFile(input_path) => {
            match output.kind {
                OutputKind::OrdinaryFile(output_path) => {
                    fs::copy(input_path, output_path).unwrap();
                }
                _ => {
                    todo!()
                }
            };
        }
        InputKind::Url(_url) => {
            match output.kind {
                OutputKind::OrdinaryFile(output_path) => {
                    download_file(_url.as_ref(), &output_path).await.unwrap();
                }
                _ => {
                    todo!()
                }
            };
        }
        InputKind::S3Bucket(bucket) => {
            match output.kind {
                OutputKind::OrdinaryFile(output_path) => {
                    download_from_s3(bucket.as_ref(), &output_path)
                        .await
                        .unwrap();
                }
                _ => {
                    todo!()
                }
            };
        }
        InputKind::StdIn => {
            match output.kind {
                OutputKind::OrdinaryFile(output_path) => {
                    file_from_std_in(&output_path).await.unwrap();
                }
                _ => {
                    todo!()
                }
            };
        }
        InputKind::ScpSource(scp_string) => {
            match output.kind {
                OutputKind::OrdinaryFile(output_path) => {
                    file_from_scp(scp_string, &output_path).await.unwrap();
                }
                _ => {
                    todo!()
                }
            };
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Some(input) = cli.input.as_deref() {
        let input: Input = to_input(input.to_string());
        println!("{:#?}", input.kind);
        if let Some(output) = cli.output.as_deref() {
            let output: Output = to_output(output.to_string());
            println!("{:#?}", output.kind);
            copy(input, output).await;
        } else {
            println!("No output defined");
        }
    } else {
        println!("No input defined");
    }
    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_input_s3() {
        let bucket = "s3://some_bucket";
        let input = to_input(bucket.to_string());
        assert_eq!(InputKind::S3Bucket(bucket.to_string()), input.kind);
    }

    #[test]
    fn test_to_input_stdin() {
        let input = to_input("-".to_string());
        assert_eq!(InputKind::StdIn, input.kind);
    }

    #[test]
    fn test_to_input_scp() {
        let source = "some_user@some_host:~/";
        let input = to_input(source.to_string());
        assert_eq!(InputKind::ScpSource(source.to_string()), input.kind);
    }

    #[test]
    fn test_to_input_file() {
        let source = "/some/path/file.txt";
        let input = to_input(source.to_string());
        assert_eq!(InputKind::OrdinaryFile(source.to_string()), input.kind);
    }

    #[test]
    fn test_to_input_url() {
        let source = "http://some_site.com/some/path/";
        let input = to_input(source.to_string());
        assert_eq!(InputKind::Url(Url::parse(source).unwrap()), input.kind);
    }

    #[test]
    fn test_to_output_s3() {
        let bucket = "s3://some_bucket";
        let output = to_output(bucket.to_string());
        assert_eq!(OutputKind::S3Bucket(bucket.to_string()), output.kind);
    }

    #[test]
    fn test_to_output_stdout() {
        let output = to_output("-".to_string());
        assert_eq!(OutputKind::StdOut, output.kind);
    }

    #[test]
    fn test_to_output_scp() {
        let target = "some_user@some_host:~/";
        let output = to_output(target.to_string());
        assert_eq!(OutputKind::ScpTarget(target.to_string()), output.kind);
    }

    #[test]
    fn test_to_output_file() {
        let target = "/some/path/file.txt";
        let output = to_output(target.to_string());
        assert_eq!(OutputKind::OrdinaryFile(target.to_string()), output.kind);
    }

    #[test]
    fn test_to_output_url() {
        let target = "http://some_site.com/some/path/";
        let output = to_output(target.to_string());
        assert_eq!(OutputKind::Url(Url::parse(target).unwrap()), output.kind);
    }
}
