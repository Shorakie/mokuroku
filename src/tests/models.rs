use crate::models::anime::{Anime, Title, Format, Status, FuzzyDate, CoverImage};

#[test]
fn test_serialize_anime_model() {
    let anime = Anime {
        id: 21,
        site_url: "https://anilist.co/anime/21".to_string(),
        title: Title {
            romaji: Some("Rom".to_string()),
            english: Some("Eng".to_string()),
            native: Some("Nat".to_string()),
            user_preferred: Some("Usr pref".to_string()),
        },
        format: Format::Tv,
        status: Status::Finished,
        genres: vec!["adventure".to_string(), "fantasy".to_string()],
        start_date: FuzzyDate {
            year: Some(2020),
            month: Some(2),
            day: Some(3),
        },
        end_date: FuzzyDate {
            year: Some(2020),
            month: Some(9),
            day: Some(7),
        },
        cover_image: CoverImage {
            medium: "https://s4.anilist.co/file/anilistcdn/media/anime/cover/small/nx21-tXMN3Y20PIL9.jpg".to_string(),
            large: "https://s4.anilist.co/file/anilistcdn/media/anime/cover/medium/nx21-tXMN3Y20PIL9.jpg".to_string(),
        },
        duration: Some(23),
        episodes: Some(12),
        average_score: Some(9),
        description: "Ranking of the kings".to_string(),
    };
    
    let serialized_anime = match serde_json::to_string(&anime) {
        Ok(result) => result,
        Err(err) => panic!("couldn't serialize anime: {:#?}", err),
    };

    assert_eq!(
        &serialized_anime,
        &"{\"id\":21,\"siteUrl\":\"https://anilist.co/anime/21\",\
        \"title\":{\"romaji\":\"Rom\",\"english\":\"Eng\",\"native\":\"Nat\",\"userPreferred\":\"Usr pref\"},\
        \"format\":\"TV\",\"status\":\"FINISHED\",\"genres\":[\"adventure\",\"fantasy\"],\
        \"startDate\":{\"year\":2020,\"month\":2,\"day\":3},\
        \"endDate\":{\"year\":2020,\"month\":5,\"day\":7},\
        \"coverImage\":\
        {\"medium\":\"https://s4.anilist.co/file/anilistcdn/media/anime/cover/small/nx21-tXMN3Y20PIL9.jpg\",\
        \"large\":\"https://s4.anilist.co/file/anilistcdn/media/anime/cover/medium/nx21-tXMN3Y20PIL9.jpg\"},\
        \"duration\":23,\"episodes\":12,\"AverageScore\":9,\"description\":\"Ranking of the kings\"}".to_owned(),
        "Serialized anime doesn't match with reference");

    let deserialized_anime: Anime = match serde_json::from_str(&serialized_anime) {
        Ok(result) => result,
        Err(err) => panic!("couldn't deserialize anime: {:#?}", err),
    };

    assert_eq!(&anime, &deserialized_anime,
        "Deserialized anime doesn't match with anime");
}