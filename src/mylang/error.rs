use super::document::Position;

#[derive(Debug, PartialEq)]
pub enum ParserError {
    InvalidToken(InvalidTokenError),
    SexpParsingError(SexpParsingError),
    AstPasringError(AstPasringError),
}

impl From<InvalidTokenError> for ParserError {
    fn from(err: InvalidTokenError) -> ParserError {
        ParserError::InvalidToken(err)
    }
}

impl From<SexpParsingError> for ParserError {
    fn from(err: SexpParsingError) -> ParserError {
        ParserError::SexpParsingError(err)
    }
}

impl From<AstPasringError> for ParserError {
    fn from(err: AstPasringError) -> ParserError {
        ParserError::AstPasringError(err)
    }
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParserError::InvalidToken(err) => write!(f, "{}", err),
            ParserError::SexpParsingError(err) => write!(f, "{}", err),
            ParserError::AstPasringError(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InvalidTokenError {
    pub position: Position,
}

impl std::fmt::Display for InvalidTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid token at: {:?}", self.position)
    }
}

impl std::error::Error for InvalidTokenError {}

#[derive(Debug, PartialEq)]
pub struct SexpParsingError {
    pub msg: String,
    pub position: Position,
}

impl std::fmt::Display for SexpParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid token at: {:?}", self.position)
    }
}

impl std::error::Error for SexpParsingError {}

#[derive(Debug, PartialEq)]
pub struct AstPasringError {
    pub msg: String,
    pub position: Position,
}

impl std::fmt::Display for AstPasringError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid token at: {:?}", self.position)
    }
}

impl std::error::Error for AstPasringError {}
