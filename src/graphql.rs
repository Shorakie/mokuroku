// Queries to use in requests
pub const SEARCH_ANIME_QUERY: &str = "
query ($search_string: String) {
  anime: Media (search: $search_string, type: ANIME) {
    id,
    siteUrl,
    title {romaji,english,native,userPreferred,},
    format,
    status(version:2),
    genres,
    startDate {year,month,day,},
    endDate {year,month,day,},
    coverImage {medium,large,},
    duration,
    episodes,
    averageScore,
    description(asHtml:true),
  }
}";

pub fn make_anime_fragment(index: u64, anime_id: i64) -> String {
  format!("anime{}: Media (id: {}, type: ANIME) {{id,title {{romaji,english,native,userPreferred}}}}", index, anime_id)
}

pub fn make_anime_query(anime_ids: &Vec<i64>) -> String {
  let mut index = 0u64;
  format!("query {{
    {}
  }}",
  anime_ids.iter().map(|anime_id| {
    let fragment = make_anime_fragment(index, *anime_id);
    index += 1;
    fragment
  }).collect::<Vec<String>>().join(","))
}