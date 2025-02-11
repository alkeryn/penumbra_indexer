pub type BoxRes<T> = Result<T, Box<dyn std::error::Error>>;
// TODO later use an error wrapper struct
