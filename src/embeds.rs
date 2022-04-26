use serenity::{builder::CreateEmbed, model::Timestamp};

pub async fn make_blank_embed(
    build: impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
) -> CreateEmbed {
    let mut e = CreateEmbed::default();
    e.timestamp(Timestamp::now());
    e.colour(0x95E1D3);
    build(&mut e);
    e
}

pub async fn make_error_embed(
    build: impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
) -> CreateEmbed {
    let mut e = make_blank_embed(|e| e).await;
    e.colour(0xF38181);
    build(&mut e);
    e
}

pub async fn make_success_embed(
    build: impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
) -> CreateEmbed {
    let mut e = make_blank_embed(|e| e).await;
    e.colour(0xB4FF9F);
    build(&mut e);
    e
}

pub async fn make_warn_embed(
    build: impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
) -> CreateEmbed {
    let mut e = make_blank_embed(|e| e).await;
    e.colour(0xFCE38A);
    build(&mut e);
    e
}
