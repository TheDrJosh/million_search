use proto::{
    search::{SearchWebRequest, SearchWebResponse},
    tonic,
};

#[derive(Debug)]
pub struct SearchServise {}

#[tonic::async_trait]
impl proto::search::search_server::Search for SearchServise {
    async fn search_web(
        &self,
        request: tonic::Request<SearchWebRequest>,
    ) -> std::result::Result<tonic::Response<SearchWebResponse>, tonic::Status> {
        todo!()
    }
}
