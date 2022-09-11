///
/// # ProPresenter 7 API
///
/// Implements a Minimal API Client using @link restson,
/// to access the current active Presentation and
///
use restson::{Error, RestPath};
use serde::{Deserialize, Deserializer, Serialize};

/// ID data of an GlobalGroup or Presentation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PP7Id {
    /// unique id inside pp7
    pub uuid: String,
    /// readable name
    pub name: String,
    pub index: u32,
}

/// ProPresenter 7 Color DataType uses f32 (0.0-1.0) and alpha channel
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PP7Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

/// Global defined Group
/// Possible Group for a Presentation Slide
/// ---
/// Contains the color for the group
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct GlobalGroup {
    id: PP7Id,
    color: PP7Color,
}

/// One Slice of a Presentation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PresentationSlide {
    enabled: bool,
    notes: String,
    text: String,
    label: String,
}

/// ## Slide Group of an Presentation
/// Contains multiple slides& a group color.
///
/// But only contains a String to identify itself
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PresentationGroup {
    pub name: String,
    #[serde(deserialize_with = "parse_color_def")]
    /// if defaults to gray if the color is null
    pub color: PP7Color,
    pub slides: Vec<PresentationSlide>,
}

fn parse_color_def<'de, D>(d: D) -> Result<PP7Color, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| {
        x.unwrap_or(PP7Color {
            alpha: 1.0,
            red: 0.5,
            green: 0.5,
            blue: 0.5,
        })
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Presentation {
    pub id: PP7Id,
    pub groups: Vec<PresentationGroup>,
    pub has_timeline: bool,
    pub presentation_path: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PresentationRequest {
    pub presentation: Presentation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PP7KeyBind {
    pub bind: String,
    pub key: String,
    pub num: usize,
}

// Path of the REST endpoint: e.g. http://<baseurl>/v1/presentation/active
impl RestPath<()> for PresentationRequest {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("/v1/presentation/active"))
    }
}

/// Alias for Vec<GlobalGroup> to make a API request
#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalGroupList(Vec<GlobalGroup>);

// Path of the REST endpoint: e.g. http://<baseurl>/anything
impl RestPath<()> for GlobalGroupList {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("/v1/groups"))
    }
}
