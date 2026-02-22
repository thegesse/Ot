use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("The_Quantity_Of_Whole_Existence")]
    IntType,

    #[token("A_Mere_Approximation_Of_Reality")]
    FloatType,

    #[token("Shall_Henceforth_Be_Seen_as")]
    Assignment,

    #[token("As_It_has_Been_written")]
    Semicolon,

    #[token("In_Addition_To_The_Aforementioned")]
    Comma,

    #[token("Under_The_Category_Of")]
    Colon,

    #[token("Augmented_By_The_Value_Of")]
    Plus,

    #[token("Diminished_By_The_Value_Of")]
    Minus,

    #[token("Replicated_In_Multitudes_Of")]
    Multiply,

    #[token("Distributed_In_Partitions_Of")]
    Divide,

    #[token("Within_This_Realm_Begin")]
    LBrace,

    #[token("By_Its_Edict_Conclude")]
    RBrace,

    #[token("Consider_The_Following")]
    LParen,

    #[token("Considerations_Concluded")]
    RParen,

    #[token("In_The_Event_That")]
    If,
    
    #[token("Otherwise_If_The_Previous_Was_False_Consider")]
    Elif,

    #[token("Otherwise_If_Contrary")]
    Else,

    #[token("Whilst_The_Truth_Remains")]
    While,

    #[token("Initiate_The_Recursive_Protocol_Under_The_Guise_Of")]
    For,

    #[token("While_The_Observation_Holds_True")]
    ForCondition,

    #[token("Until_The_Incremental_Shift_Reaches")]
    ForStep,

    #[token("Request_The_Universe_Manifest_The_Knowledge_Of")]
    Print,

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r#""[^"]*""#, |lex| Some(lex.slice()[1..lex.slice().len()-1].to_string()))]
    StringLiteral(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}