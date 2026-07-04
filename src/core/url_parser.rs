mod amdm;
mod akkords;
mod _5lad;


pub const AVAILABLE_SITES: [&str; 3] = [
    "amdm",
    "akkords",
    "5lad",
];


impl crate::Song {
    pub fn from_url(url: &str) ->Option<Self> {
        let stripped_url: &str = 
            if url.starts_with("https://") {
                url.strip_prefix("https://")?
            } else if url.starts_with("http://") {
                url.strip_prefix("http://")?
            } else {
                url
            };
    
    
        if stripped_url.starts_with("amdm") {
            amdm::parse(url)
        } else if stripped_url.starts_with("akkords") {
            akkords::parse(url)
        } else if stripped_url.starts_with("5lad") {
            _5lad::parse(url)
        } else {
            None
        }
    }
}
