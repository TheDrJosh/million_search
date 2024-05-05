use proto::{
    search::{
        CompleteSearchRequest, CompleteSearchResponse, SearchAudioResponse, SearchImageResponse,
        SearchRequest, SearchVideoResponse, SearchWebResponse,
    },
    tonic,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteSearch {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub text_fields: Vec<String>,
    pub sections: Vec<String>,
}

#[derive(Debug)]
pub struct SearchServise {}

#[tonic::async_trait]
impl proto::search::search_server::Search for SearchServise {
    async fn complete_search(
        &self,
        request: tonic::Request<CompleteSearchRequest>,
    ) -> std::result::Result<tonic::Response<CompleteSearchResponse>, tonic::Status> {
        let request = request.into_inner();

        todo!()
    }

    async fn search_web(
        &self,
        request: tonic::Request<SearchRequest>,
    ) -> std::result::Result<tonic::Response<SearchWebResponse>, tonic::Status> {
        let request = request.into_inner();

        todo!()
    }

    async fn search_image(
        &self,
        request: tonic::Request<SearchRequest>,
    ) -> std::result::Result<tonic::Response<SearchImageResponse>, tonic::Status> {
        let request = request.into_inner();

        todo!()
    }

    async fn search_video(
        &self,
        request: tonic::Request<SearchRequest>,
    ) -> std::result::Result<tonic::Response<SearchVideoResponse>, tonic::Status> {
        let request = request.into_inner();

        todo!()
    }
    async fn search_audio(
        &self,
        request: tonic::Request<SearchRequest>,
    ) -> std::result::Result<tonic::Response<SearchAudioResponse>, tonic::Status> {
        let request = request.into_inner();

        todo!()
    }
}
