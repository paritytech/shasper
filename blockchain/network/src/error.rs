#[derive(Debug)]
pub enum Error {
	IO(std::io::Error),
	Libp2p(Box<dyn std::error::Error + Sync + Send + 'static>),
	Other(String),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error { }

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Error {
		Error::IO(err)
	}
}

impl From<String> for Error {
	fn from(s: String) -> Error {
		Error::Other(s)
	}
}
