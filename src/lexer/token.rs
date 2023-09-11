use super::token_type::TokenType;


pub(crate) struct Token {
    pub(crate) ttype: TokenType,
    pub(crate) lexeme: String,
    pub(crate) line: usize,
    pub(crate) offset: usize,
    pub(crate) length: usize,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        let token_type = self.ttype.to_string();
        let lexeme = &self.lexeme;
        let line = self.line;
        let start = self.offset;
        let end = self.offset + self.length;
        format!("({token_type}, '{lexeme}') {line}:{start}-{end}")
    }
}
