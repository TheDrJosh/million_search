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
                    .col(ColumnDef::new(Image::SourceUrl).string().not_null())
                    .col(ColumnDef::new(Image::Width).integer())
                    .col(ColumnDef::new(Image::Height).integer())
                    .col(ColumnDef::new(Image::AltText).string())
                    .col(
                        ColumnDef::new(Image::CreatedAt)
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
    AltText,
    SourceUrl,
    CreatedAt,
}
