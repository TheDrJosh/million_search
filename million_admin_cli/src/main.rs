use clap::{Parser, Subcommand};
use proto::{
    admin::{AddUrlToQueueRequest, GetAllUrlsInQueueRequest},
    tonic::codec::CompressionEncoding,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    backend_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    AddUrl { url: String },
    GetAllUrl,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let backend_url = cli
        .backend_url
        .unwrap_or(String::from("http://localhost:8080"));

    let mut backend = proto::admin::admin_client::AdminClient::connect(backend_url)
        .await
        .unwrap()
        .send_compressed(CompressionEncoding::Zstd)
        .accept_compressed(CompressionEncoding::Zstd);

    match cli.command {
        Commands::AddUrl { url } => {
            //
            let _res = backend
                .add_url_to_queue(AddUrlToQueueRequest { url })
                .await
                .unwrap();
        }
        Commands::GetAllUrl => {
            let res = backend
                .get_all_urls_in_queue(GetAllUrlsInQueueRequest {})
                .await
                .unwrap()
                .into_inner()
                .urls;
            println!("{:#?}", res);
        }
    }
}
