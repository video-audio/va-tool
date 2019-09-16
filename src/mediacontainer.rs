use url::Url;

#[derive(Debug, Eq, PartialEq)]
pub enum Mediacontainer {
    Ts,
    Mp4 { fragmented: bool },
    WebM,
    Rtp,
    Rtsp,
}

impl From<&url::Url> for Mediacontainer {
    fn from(u: &Url) -> Self {
        Mediacontainer::Ts
    }
}
