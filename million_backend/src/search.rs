use entity::search_history;
use meilisearch_sdk::Client;
use proto::{
    search::{
        CompleteSearchRequest, CompleteSearchResponse, SearchAudioRequest, SearchAudioResponse,
        SearchImageRequest, SearchImageResponse, SearchVideoRequest, SearchVideoResponse,
        SearchWebRequest, SearchWebResponse,
    },
    tonic::{self, Status},
};
use sea_orm::{ActiveModelTrait, DatabaseConnection};
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
pub struct SearchServise {
    pub db: DatabaseConnection,
    pub search_client: Client,
}

#[tonic::async_trait]
impl proto::search::search_server::Search for SearchServise {
    async fn complete_search(
        &self,
        request: tonic::Request<CompleteSearchRequest>,
    ) -> std::result::Result<tonic::Response<CompleteSearchResponse>, tonic::Status> {
        let request = request.into_inner();

        // let results: SearchResults<Movie> = client
        //     .index("movies")
        //     .search()
        //     .with_query("botman")
        //     .execute()
        //     .await
        //     .unwrap();

        todo!()
    }

    async fn search_web(
        &self,
        request: tonic::Request<SearchWebRequest>,
    ) -> std::result::Result<tonic::Response<SearchWebResponse>, tonic::Status> {
        let request = request.into_inner();

        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query)
            .await
            .map_err(|err| Status::from_error(err.into()))?;

        // let result: SearchResults<> = self
        //     .search_client
        //     .index("websites")
        //     .search()
        //     .with_query(&query.query)
        //     .execute()
        //     .await
        //     .map_err(|err| Status::from_error(err.into()))?;

        todo!()
    }

    async fn search_image(
        &self,
        request: tonic::Request<SearchImageRequest>,
    ) -> std::result::Result<tonic::Response<SearchImageResponse>, tonic::Status> {
        let request = request.into_inner();
        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query)
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        todo!()
    }

    async fn search_video(
        &self,
        request: tonic::Request<SearchVideoRequest>,
    ) -> std::result::Result<tonic::Response<SearchVideoResponse>, tonic::Status> {
        let request = request.into_inner();
        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query)
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        todo!()
    }
    async fn search_audio(
        &self,
        request: tonic::Request<SearchAudioRequest>,
    ) -> std::result::Result<tonic::Response<SearchAudioResponse>, tonic::Status> {
        let request = request.into_inner();
        let query = request
            .query
            .ok_or(Status::invalid_argument("must have query"))?;

        save_search_to_history(&self.db, query.query)
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        todo!()
    }
}

async fn save_search_to_history(db: &DatabaseConnection, search: String) -> anyhow::Result<()> {
    let search = search_history::ActiveModel {
        text: sea_orm::ActiveValue::Set(search),
        ..Default::default()
    };
    search.insert(db).await?;
    Ok(())
}

// struct Websites {
//     id: i64,

// }
