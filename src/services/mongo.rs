use std::env;
use mongodb::{bson::{doc, to_bson, Document}, Client, Collection, options::{ClientOptions, UpdateOptions}};
use bson::DateTime;
use serenity::{
    model::id::UserId,
    futures::StreamExt,
};
use chrono::{Utc};
use tracing::{info, error};

async fn get_collection(collection_name: &str) -> mongodb::error::Result<Collection<Document>> {
    let mongo_uri = env::var("MONGODB_URI").expect("Expected a MONGO_URI in the environment");
    let mongo_options = ClientOptions::parse(mongo_uri).await?;
    let client = Client::with_options(mongo_options)?;
    Ok(client.database("mokuroku").collection::<Document>(collection_name))
}

pub async fn set_watching_status(user_id: UserId, anime_id: i64, status: &str) -> mongodb::error::Result<()> {
    let watch_list = get_collection("watch-list").await?;
    
    let filter_query = doc! {
        "anime_id": anime_id,
        "user_id": to_bson(user_id.as_u64()).unwrap(),
    };

    let to_watch = doc! {
        "$set": {
            "anime_id": anime_id,
            "user_id": to_bson(user_id.as_u64()).unwrap(),
            "status": status,
            "update_at": DateTime::from_chrono(Utc::now()),
        }
    };

    info!("inserting or updating {:?} with {:?}", &filter_query, &to_watch);
    match watch_list.update_one(filter_query.clone(), to_watch.clone(), UpdateOptions::builder().upsert(true).build()).await {
        Ok(_) => info!("inserted/updated {:?}", &to_watch),
        Err(why) => error!("error while inserting/updating: {:#?}", why),
    };

    Ok(())
}

pub async fn remove_watching(user_id: UserId, anime_id: i64) -> mongodb::error::Result<()> {
    let watch_list = get_collection("watch-list").await?;

    let filter_query = doc! {
        "anime_id": anime_id,
        "user_id": to_bson(user_id.as_u64()).unwrap(),
    };

    info!("removing {:?}", &filter_query);
    match watch_list.delete_one(filter_query.clone(), None).await {
        Ok(_) => info!("removed {:?}", &filter_query),
        Err(why) => error!("error while removing: {:#?}", why),
    };


    Ok(())
}

pub async fn suggest(user_id: UserId, anime_id: i64) -> mongodb::error::Result<()> {
    let suggest_list = get_collection("recommendation-list").await?;

    let filter_query = doc! {
        "anime_id": anime_id,
        "user_id": to_bson(user_id.as_u64()).unwrap(),
    };

    let to_suggest = doc! {
        "$set": {
            "anime_id": anime_id,
            "user_id": to_bson(user_id.as_u64()).unwrap(),
            "update_at": DateTime::from_chrono(Utc::now()),
        }
    };

    info!("inserting or updating {:?} with {:?}", &filter_query, &to_suggest);
    match suggest_list.update_one(filter_query.clone(), to_suggest.clone(), UpdateOptions::builder().upsert(true).build()).await {
        Ok(_) => info!("inserted/updated {:?}", &to_suggest),
        Err(why) => error!("error while inserting/updating: {:#?}", why),
    };

    Ok(())
}

pub async fn remove_suggest(user_id: UserId, anime_id: i64) -> mongodb::error::Result<()> {
    let suggest_list = get_collection("recommendation-list").await?;
    let filter_query = doc! {
        "anime_id": anime_id,
        "user_id": to_bson(user_id.as_u64()).unwrap(),
    };

    info!("removing {:?}", &filter_query);
    match suggest_list.delete_one(filter_query.clone(), None).await {
        Ok(_) => info!("removed {:?}", &filter_query),
        Err(why) => error!("error while removing: {:#?}", why),
    };

    Ok(())
}

pub async fn get_suggesting(user_id: UserId) -> mongodb::error::Result<Vec<i64>> {
    let suggest_list = get_collection("recommendation-list").await?;
    let pipeline = vec![
        doc! { "$match": {
            "user_id": to_bson(user_id.as_u64()).unwrap(),
        }},
        doc! { "$group": {
            "_id": "$user_id",
            "anime_ids": {"$push": "$anime_id"},
        }},
        doc! { "$project": {
            "_id": false,
            "anime_ids": true,
        }},
    ];

    let mut suggestions: Vec<i64> = vec![];
    let mut results = match suggest_list.aggregate(pipeline, None).await {
        Ok(results) => results,
        Err(why) => {
            error!("while aggregating: {:#?}", why);
            return Ok(suggestions);
        }
    };
    while let Some(result) = results.next().await {
        // Use serde to deserialize into the MovieSummary struct:
        let result = result?;
        match result.get_array("anime_ids") {
            Ok(anime_ids) => {
                let mut anime_ids: Vec<i64> = anime_ids.iter().filter_map(|id| id.as_i64()).collect();
                suggestions.append(&mut anime_ids);
                info!("entry: {:?}", suggestions);
            },
            Err(why) => error!("couldn't read anime_ids: {:#?}", why),
        }
        // suggestions.push(result?);
    }
    Ok(suggestions)
}

pub async fn get_watching(user_id: UserId, statuses: Option<Vec<&str>>) -> mongodb::error::Result<Vec<i64>> {
    let watch_list = get_collection("watch-list").await?;
    info!("Got the collection!");
    let pipeline = vec![
        doc! { "$match": {
            "user_id": to_bson(user_id.as_u64()).unwrap(),
            "status": {"$in": statuses.unwrap_or(vec!["FINISHED", "WATCHING"])},
        }},
        doc! { "$group": {
            "_id": "$user_id",
            "anime_ids": {"$push": "$anime_id"},
        }},
        doc! { "$project": {
            "_id": false,
            "anime_ids": true,
        }},
    ];

    let mut watching: Vec<i64> = vec![];
    let mut results = match watch_list.aggregate(pipeline, None).await {
        Ok(results) => results,
        Err(why) => {
            error!("while aggregating: {:#?}", why);
            return Ok(watching);
        }
    };
    while let Some(result) = results.next().await {
        // Use serde to deserialize into the MovieSummary struct:
        let result = result?;
        match result.get_array("anime_ids") {
            Ok(anime_ids) => {
                let mut anime_ids: Vec<i64> = anime_ids.iter().filter_map(|id| id.as_i64()).collect();
                watching.append(&mut anime_ids);
                info!("entry: {:?}", watching);
            },
            Err(why) => error!("couldn't read anime_ids: {:#?}", why),
        }
        // watching.push(result?);
    }
    Ok(watching)
}