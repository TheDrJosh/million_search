use std::pin::Pin;

use proto::crawler::{StreamJobsRequest, StreamJobsResponse};
use tokio_stream::Stream;
use tonic::{Status, Streaming};

#[derive(Debug, Default)]
pub struct CrawlerServise {}

#[tonic::async_trait]
impl proto::crawler::crawler_server::Crawler for CrawlerServise {
    type StreamJobsStream = Pin<Box<dyn Stream<Item = Result<StreamJobsResponse, Status>> + Send>>;

    async fn stream_jobs(
        &self,
        request: tonic::Request<Streaming<StreamJobsRequest>>,
    ) -> std::result::Result<tonic::Response<Self::StreamJobsStream>, tonic::Status> {
        todo!()
    }
}
