pub type IndexerResult<T> = Result<T, ErrorWrapper>;

#[derive(Debug)]
pub struct ErrorWrapper {
    source: ErrorKind
}

impl std::fmt::Display for ErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for ErrorWrapper {}

macro_rules! impl_errorkind {
    (
        $($name:ident($err_type:ty)),*;
        $($unimpl:ident($err_type_unimpl:ty)),*
    ) => {
        #[derive(Debug)]
        pub enum ErrorKind {
            $($name($err_type)),*

        }
        $(
            impl From<$err_type> for ErrorWrapper {
                fn from(value: $err_type) -> Self {
                    ErrorWrapper {
                        source: ErrorKind::$name(value)
                    }
                }
            }
        )*
    };
    (
        $($name:ident($err_type:ty)),*
    ) => {
        impl_errorkind!(
            $($name($err_type)),*
            ;
        );
    }
}


impl_errorkind!(
    Tonic(tonic::Status),
    TonicTransport(tonic::transport::Error),
    SerdeJson(serde_json::Error),
    DecodeError(prost::DecodeError)
);

// TODO later use an error wrapper struct
