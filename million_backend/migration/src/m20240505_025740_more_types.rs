use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Image::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Image::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Image::Url).string().not_null())
                    .col(ColumnDef::new(Image::Width).integer())
                    .col(ColumnDef::new(Image::Height).integer())
                    .col(
                        ColumnDef::new(Image::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Video::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Video::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Video::Url).string().not_null())
                    .col(ColumnDef::new(Video::Width).integer().not_null())
                    .col(ColumnDef::new(Video::Height).integer().not_null())
                    .col(ColumnDef::new(Video::LengthMillis).integer().not_null())
                    .col(
                        ColumnDef::new(Video::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Audio::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Audio::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Audio::Url).string().not_null())
                    .col(
                        ColumnDef::new(Audio::LengthMillis)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Audio::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Video::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Audio::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Image {
    Table,
    Id,
    Url,
    Width,
    Height,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Video {
    Table,
    Id,
    Url,
    Width,
    Height,
    LengthMillis,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Audio {
    Table,
    Id,
    Url,
    LengthMillis,
    CreatedAt,
}
