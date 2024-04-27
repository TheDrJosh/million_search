use std::{error::Error, io::ErrorKind, pin::Pin};

use proto::crawler::{StreamJobsRequest, StreamJobsResponse};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Response, Status, Streaming};

#[derive(Debug, Default)]
pub struct CrawlerServise {}

#[tonic::async_trait]
impl proto::crawler::crawler_server::Crawler for CrawlerServise {
    type StreamJobsStream = Pin<Box<dyn Stream<Item = Result<StreamJobsResponse, Status>> + Send>>;

    async fn stream_jobs(
        &self,
        request: tonic::Request<Streaming<StreamJobsRequest>>,
    ) -> std::result::Result<tonic::Response<Self::StreamJobsStream>, tonic::Status> {
        let mut in_stream = request.into_inner();

        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => {
                        tx.send(
                            handle_request(v)
                                .await
                                .map_err(|err| Status::from_error(err.into())),
                        )
                        .await
                        .unwrap();
                    }
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                // handle sudden disconnect
                            }
                        }

                        match tx.send(Err(err)).await {
                            Ok(_) => (),
                            Err(_) => break,
                        }
                    }
                };
            }
        });

        let out_stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(out_stream) as Self::StreamJobsStream))
    }
}

pub async fn handle_request(req: StreamJobsRequest) -> anyhow::Result<StreamJobsResponse> {
    todo!()
}

fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        // h2::Error do not expose std::io::Error with `source()`
        // https://github.com/hyperium/h2/pull/462
        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }

        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}
