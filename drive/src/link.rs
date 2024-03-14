use url::Url;

#[derive(Debug)]
pub struct Link {
    pub url: String,
    pub id: String,
}

impl Link {
    pub fn new(url: String) -> Self {
        let parsed_url = Url::parse(url.as_str());
        let mut id = url.clone();

        if parsed_url.is_ok() {
            let unwrapped_parsed_url = parsed_url.unwrap();
            let path_split = unwrapped_parsed_url
                .path_segments()
                .map(|c| c.collect::<Vec<_>>())
                .unwrap();

            if path_split.len() >= 3 {
                id = path_split.get(2).unwrap().to_string();
            }
        }

        Self {
            url: url.to_string(),
            id,
        }
    }
}
