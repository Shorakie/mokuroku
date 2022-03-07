use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum Format {
    #[serde(rename = "TV")]
    Tv,
    #[serde(rename = "TV_SHORT")]
    TvShort,
    #[serde(rename = "MOVIE")]
    Movie,
    #[serde(rename = "SPECIAL")]
    Special,
    #[serde(rename = "OVA")]
    Ova,
    #[serde(rename = "ONA")]
    Ona,
    #[serde(rename = "MUSIC")]
    Music,
    #[serde(rename = "MANGA")]
    Manga,
    #[serde(rename = "NOVEL")]
    Novel,
    #[serde(rename = "ONE_SHOT")]
    OneShot,
}

impl ToString for Format {
    #[inline]
    fn to_string(&self) -> String {
        match self {
            &Self::Tv => "📺 Tv".to_owned(),
            &Self::TvShort => "📺 Tv Short".to_owned(),
            &Self::Movie => "🎥 Movie".to_owned(),
            &Self::Special => "🌟 Special".to_owned(),
            &Self::Ova => "💽 OVA".to_owned(),
            &Self::Ona => "💻 ONA".to_owned(),
            &Self::Music => "🎵 Music".to_owned(),
            &Self::Manga => "📔 Manga".to_owned(),
            &Self::Novel => "📙 Novel".to_owned(),
            &Self::OneShot => "📄 One Shot".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum Status {
    #[serde(rename = "FINISHED")]
    Finished,
    #[serde(rename = "RELEASING")]
    Releasing,
    #[serde(rename = "NOT_YET_RELEASED")]
    NotYetReleased,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "HIATUS")]
    Hiatus,
}

impl ToString for Status {
    #[inline]
    fn to_string(&self) -> String {
        match self {
            &Self::Finished => "Finished".to_owned(),
            &Self::Releasing => "Releasing".to_owned(),
            &Self::NotYetReleased => "Not Released Yet".to_owned(),
            &Self::Canceled => "Canceled".to_owned(),
            &Self::Hiatus => "Hiatus".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct FuzzyDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

impl ToString for FuzzyDate {
    #[inline]
    fn to_string(&self) -> String {
        self.year.and(self.month).and(self.day).map_or(
            "?".to_owned(),
            |_| format!("{}-{}-{}", &self.year.unwrap(), &self.month.unwrap(), &self.day.unwrap()))
    }
}


#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct CoverImage {
    pub medium: String,
    pub large: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Anime {
    pub id: i32,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub title: Title,
    pub format: Format,
    pub status: Status,
    pub genres: Vec<String>,
    #[serde(rename = "startDate")]
    pub start_date: FuzzyDate,
    #[serde(rename = "endDate")]
    pub end_date: FuzzyDate,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    pub duration: Option<i32>,
    pub episodes: Option<i32>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    pub description: String,
}

impl Anime {
    pub fn get_title(&self) -> String {
        self.title.user_preferred.as_ref()
        .or(self.title.english.as_ref())
        .or(self.title.romaji.as_ref())
        .or(self.title.native.as_ref())
        .unwrap_or(&"?".to_string())
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryError {
    pub message: String,
    pub status: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SingleAnimeData {
    pub anime: Anime
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchAnimeData {
    pub anime0: Option<Anime>,
    pub anime1: Option<Anime>,
    pub anime2: Option<Anime>,
    pub anime3: Option<Anime>,
    pub anime4: Option<Anime>,
    pub anime5: Option<Anime>,
    pub anime6: Option<Anime>,
    pub anime7: Option<Anime>,
    pub anime8: Option<Anime>,
    pub anime9: Option<Anime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResponse<T> {
    pub errors: Option<Vec<QueryError>>,
    pub data: Option<T>,
}

pub type SingleAnimeQueryResponse = QueryResponse<SingleAnimeData>;
pub type BatchAnimeQueryResponse = QueryResponse<BatchAnimeData>;