pub mod frontend {
    tonic::include_proto!("search");
}

pub mod admin {
    tonic::include_proto!("admin");
}

pub mod crawler {
    tonic::include_proto!("crawler");
}
