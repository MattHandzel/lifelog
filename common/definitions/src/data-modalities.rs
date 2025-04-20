pub enum Data {
    Image(image::DynamicImage),
    Text(String),
}

trait DataType {
    fn table_name() -> &'static str;
    fn schema() -> 
}
