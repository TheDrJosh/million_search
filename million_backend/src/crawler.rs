use proto::crawler::{
    GetJobRequest, GetJobResponse, KeepAliveJobRequest, KeepAliveJobResponse, ReturnJobRequest,
    ReturnJobResponse,
};

#[derive(Debug, Default)]
pub struct CrawlerServise {}

#[tonic::async_trait]
impl proto::crawler::crawler_server::Crawler for CrawlerServise {
    async fn get_job(
        &self,
        request: tonic::Request<GetJobRequest>,
    ) -> std::result::Result<tonic::Response<GetJobResponse>, tonic::Status> {
        todo!()
    }

    async fn return_job(
        &self,
        request: tonic::Request<ReturnJobRequest>,
    ) -> std::result::Result<tonic::Response<ReturnJobResponse>, tonic::Status> {
        todo!()
    }

    async fn keep_alive_job(
        &self,
        request: tonic::Request<KeepAliveJobRequest>,
    ) -> std::result::Result<tonic::Response<KeepAliveJobResponse>, tonic::Status> {
        todo!()
    }
}
