use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Exclamation /* ! */, At           /* @ */, Hash     /* # */, Dollar     /* $ */, Percent  /* % */,
    Caret       /* ^ */, Ampersand    /* & */, Asterisk /* * */, Number     /* № */, Question /* ? */,
    Plus        /* + */, Equal        /* = */, Minus    /* - */, Underscore /* _ */,
    LeftParen   /* ( */, RightParen   /* ) */, Comma    /* , */, Dot        /* . */,
    LeftBracket /* [ */, RightBracket /* ] */, Slash    /* / */, BackSlash  /* \ */,
    LeftBrace   /* { */, RightBrace   /* } */, Pipe     /* | */,
    LessThan    /* < */, GreatThan    /* > */, Colon    /* : */, Semicolon  /* ; */,
    Grave       /* ` */, Tilda        /* ~ */, Quote    /* " */, Apostrophe /* ' */,
}

impl FromStr for Symbol {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "!" => Self::Exclamation,
            "@" => Self::At,
            "#" => Self::Hash,
            "$" => Self::Dollar,
            "%" => Self::Percent,
            "^" => Self::Caret,
            "&" => Self::Ampersand,
            "*" => Self::Asterisk,
            "№" => Self::Number,
            "?" => Self::Question,
            "+" => Self::Plus,
            "=" => Self::Equal,
            "-" => Self::Minus,
            "_" => Self::Underscore,
            "(" => Self::LeftParen,
            ")" => Self::RightParen,
            "," => Self::Comma,
            "." => Self::Dot,
            "[" => Self::LeftBracket,
            "]" => Self::RightBracket,
            "/" => Self::Slash,
            "\\" => Self::BackSlash,
            "{" => Self::LeftBrace,
            "}" => Self::RightBrace,
            "|" => Self::Pipe,
            "<" => Self::LessThan,
            ">" => Self::GreatThan,
            ":" => Self::Colon,
            ";" => Self::Semicolon,
            "`" => Self::Grave,
            "~" => Self::Tilda,
            "\"" => Self::Quote,
            "'" => Self::Apostrophe,
            _ => return Err(()),
        })
    }
}
