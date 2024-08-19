use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = String::from("0.0.0.0:8443"))]
    pub addr: String,
    #[arg(long, default_value_t = String::from("./server.crt"))]
    pub cert: String,
    #[arg(long, default_value_t = String::from("./server.key"))]
    pub key: String,
    #[arg(short, long, default_value_t = String::from("info"))]
    pub log_level: String,
    #[arg(short, long, default_value_t = 10)]
    pub duration: i32,
}
