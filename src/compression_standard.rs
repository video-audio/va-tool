#[derive(Debug, Eq, PartialEq)]
pub enum Video {
    Mpeg2,
    Mpeg4,
    H264,
    H265,
    Vp8,
    Vp9,
    AV1,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CompressionStandard {
    Video,
    Audio,
    Image,
    Subtitle,
    Cc,
    Teletext,
}
