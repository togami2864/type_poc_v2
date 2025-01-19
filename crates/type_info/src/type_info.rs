#[derive(Debug, Clone)]
pub enum TypeInfo {
    Keyword(TsKeywordType),
    Interface {
        name: String,
        properties: Vec<(String, TypeInfo)>,
    },
    TypeAlias {
        name: String,
        aliased_type: Box<TypeInfo>,
    },
    Literal {
        value: String,
    },
    Function {
        params: Vec<(String, TypeInfo)>,
        return_type: Box<TypeInfo>,
    },
    Reference(String),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum TsKeywordType {
    TSAnyKeyword,
    TSBigIntKeyword,
    TSBooleanKeyword,
    TSNeverKeyword,
    TSNullKeyword,
    TSNumberKeyword,
    TSObjectKeyword,
    TSStringKeyword,
    TSSymbolKeyword,
    TSUndefinedKeyword,
    TSUnknownKeyword,
    TSVoidKeyword,
}
