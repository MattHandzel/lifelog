pub mod models;

// This enum will define what the underlying information is, this is analagous to where it comes
// from, for example, images come from the camera, audio comes from the microphone, etc.
enum InformationType {
    Text,
    Image,
    Audio,
}
