use super::ast::*;

use pom::utf8::*;

use std::str::FromStr;

// ****************************************************************************
// Pom representation of "The Assembler" section of:
// https://github.com/nigeleke/bbc-x/blob/main/docs/BBCX.pdf
//
// The assember is single pass so that each line of source code
// is translated into object code completely on a line by line
// basis.

pub struct Grammar;

impl Grammar {
    pub fn bbcx_line<'a>() -> Parser<'a, SourceLine> {
        source_program_line()
    }
}

// ****************************************************************************
// A source program line consists of the address of the location which is to
// receive the translated version of the source word followed by the source
// word itself.
fn source_program_line<'a>() -> Parser<'a, SourceLine> {
    let as_source_program_line = |(((l, lbl), w), c)| SourceLine::new(l, lbl, w, c);

    (location() + label() - inline_ws().opt() + source_program_word() - inline_ws().opt()
        + comment()
        - end())
    .map(as_source_program_line)
    .name("source_program_line")
}

// <location> ::= <numeric address><space>
fn location<'a>() -> Parser<'a, Location> {
    (absolute_address() - inline_ws()).name("location")
}

// label := <idenifier> :
fn label<'a>() -> Parser<'a, Label> {
    ((identifier() - sym(':')).map(Label::from)) | empty().map(|_| Label::from(None)).name("label")
}

// This source word can be any of the four tyoes (but certain conventions
// must be kept to if the program is to list correctly)
fn source_program_word<'a>() -> Parser<'a, SourceWord> {
    (pword().map(SourceWord::PWord)
        | fword().map(SourceWord::FWord)
        | iword().map(SourceWord::IWord)
        | sword().map(SourceWord::SWord))
    .name("source_program_word")
}

// ; COMMENT
fn comment<'a>() -> Parser<'a, Comment> {
    let actual = || {
        sym(';')
            * none_of("\n")
                .repeat(0..)
                .map(|s| format!(";{}", String::from_iter(s)))
    };
    (actual() | empty().map(|_| "".to_string())).name("comment")
}

// ****************************************************************************
// The basic format for a P-word consists of a mnemonic terminated by a space
// followed by an accumulator address (an integer from 0 to 7) terminated by a
// comma followed by a store operand specifier.
fn pword<'a>() -> Parser<'a, PWord> {
    (mnemonic() - inline_ws().opt() + acc() - inline_ws().opt() + store_operand())
        .map(|((m, a), so)| PWord::new(m, a, so))
        .name("pword")
}

// ****************************************************************************
// <S-word> ::= <quote><actual character><character><character>
//              <character><unquote>
fn sword<'a>() -> Parser<'a, SWord> {
    (sym('"') + actual_character().repeat(1..=4) + sym('"'))
        .map(|((_, cs), _)| SWord::from_iter(cs.iter()))
        .name("sword")
}
// <F-word> ::= <unsigned F-word> | <signed F-word>
fn fword<'a>() -> Parser<'a, FWord> {
    (unsigned_fword() | signed_fword()).name("fword")
}

// <I-word> ::= <unsigned integer> | <signed integer>
fn iword<'a>() -> Parser<'a, IWord> {
    (unsigned_integer() | signed_integer()).name("iword")
}

// <digit> ::= <oct.dig> | 8 | 9
fn digit<'a>() -> Parser<'a, char> {
    oct_dig() | one_of("89")
}

// <oct.dig> ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
fn oct_dig<'a>() -> Parser<'a, char> {
    one_of(&('0'..='7'))
}

// Added: ++
// Represents 'space()' in description
fn inline_ws<'a>() -> Parser<'a, String> {
    one_of(" \t")
        .repeat(1..)
        .map(String::from_iter)
        .name("inline_ws")
}
// Added: --

fn store_operand<'a>() -> Parser<'a, StoreOperand> {
    (address_operand().map(StoreOperand::AddressOperand)
        | const_operand().map(StoreOperand::ConstOperand)
        | empty().map(|_| StoreOperand::None))
    .name("store_operand")
}

fn address_operand<'a>() -> Parser<'a, AddressOperand> {
    (simple_address_operand() + index_ref().opt())
        .map(|(o, i)| AddressOperand::new(o, i))
        .name("address_operand")
}

// <const. operand> ::= <signed integer> | <signed F-word> !
//                      <octal> | <S-word>
fn const_operand<'a>() -> Parser<'a, ConstOperand> {
    inline_ws().opt()
        * (signed_fword().map(ConstOperand::SignedFWord)
            | signed_integer().map(ConstOperand::SignedIWord)
            | sword().map(ConstOperand::SWord))
        .name("const_operand")
}

fn simple_address_operand<'a>() -> Parser<'a, SimpleAddressOperand> {
    (address().map(SimpleAddressOperand::DirectAddress)
        | (sym('*') * address()).map(SimpleAddressOperand::IndirectAddress))
    .name("simple_address_operand")
}

// <address> ::= <identifier> | <numeric address>
fn address<'a>() -> Parser<'a, Address> {
    (identifier().map(Address::Identifier) | numeric_address().map(Address::NumericAddress))
        .name("address")
}

// <identifier> ::= <alpha char.> | <identifier><alpha char.> !
//                  <identifier><digit>
fn identifier<'a>() -> Parser<'a, Identifier> {
    let concat = |a: char, ans: &[char]| {
        let mut id = Vec::from(ans);
        id.insert(0, a);
        String::from_iter(id)
    };

    (alpha_character() + alpha_numeric().repeat(0..))
        .map(move |ans| concat(ans.0, &ans.1))
        .name("identifier")
}

// <actual character> ::= <alpha character> | <numeric character> |
//                        <punctuation>
fn actual_character<'a>() -> Parser<'a, Character> {
    (alpha_character() | numeric_character() | punctuation()).name("actual_character")
}

// <alpha character> ::= A | B | C | D | E | F | G | H | I | J | K| L |M | N | O | P |
//                       Q | R | S | T | V | W | X | Y | Z
fn alpha_character<'a>() -> Parser<'a, char> {
    one_of(&('A'..='Z')).name("alpha_character")
}

// <numeric character> ::= <digit> | + | - | <subscript 10> | .
fn numeric_character<'a>() -> Parser<'a, NumericCharacter> {
    (digit() | sym('+') | sym('-') | subscript10() | sym('.')).name("numeric_character")
}

// <punctuation> ::= ( | <quote> | <unquote> | <apostrophe> | * | / |
//                   : | ) | = | ? | ^ | ~ | # | ; | , | <space>
fn punctuation<'a>() -> Parser<'a, Punctuation> {
    one_of("<>'*/:)=?^~#;. ").name("punctuation")
}

fn alpha_numeric<'a>() -> Parser<'a, char> {
    (alpha_character() | digit()).name("alpha_numeric")
}

fn subscript10<'a>() -> Parser<'a, char> {
    sym('@')
}

// <numeric address> ::= <absolute address> | <relative address>
fn numeric_address<'a>() -> Parser<'a, NumericAddress> {
    unsigned_integer()
        .map(|i| i as NumericAddress)
        .name("numeric_address")
}

// TODO: Work out what an Index actually is in bbc-x; it appears to be numeric only and possibly single digit??
fn index<'a>() -> Parser<'a, Index> {
    digit()
        .repeat(1..=2)
        .map(String::from_iter)
        .convert(|s| Index::from_str(&s))
        .name("index")
}

fn index_ref<'a>() -> Parser<'a, Index> {
    (sym('[') * index() - sym(']')).map(|i| i).name("index_ref")
}

// ****************************************************************************
// <signed F-word> ::= +<unsigned F-word> | -<unsigned F-word>
fn signed_fword<'a>() -> Parser<'a, FWord> {
    let negate = |f: FloatType| -f;

    ((sym('+') * unsigned_fword()) | (sym('-') * unsigned_fword().map(negate))).name("signed_fword")
}

// <unsigned F-word> ::= <decimal part> | <decimal part>
//                       <exponent part> | <unsigned integer>
//                       <exponent part>
fn unsigned_fword<'a>() -> Parser<'a, FWord> {
    let de_to_float1 = |(d, e)| FloatType::from_str(&format!("{}e{}", d, e));
    let de_to_float2 = |(d, e)| FloatType::from_str(&format!("{}e{}", d, e));

    (decimal_part().convert(|s| FloatType::from_str(&s))
        | (decimal_part() + exponent_part()).convert(de_to_float1)
        | (unsigned_integer() + exponent_part()).convert(de_to_float2))
    .name("unsigned_fword")
}

// <decimal part> ::= <unsigned integer>.<unsigned integer> |
//                    .<unsigned integer>
fn decimal_part<'a>() -> Parser<'a, String> {
    let if_to_string = |(i, f): (IntType, IntType)| format!("{}.{}", i, f);
    let f_to_string = |f: IntType| format!("0.{}", f);

    ((unsigned_integer() - sym('.').discard() + unsigned_integer()).map(if_to_string)
        | (sym('.').discard() * unsigned_integer()).map(f_to_string))
    .name("decimal_part")
}

// <exponent part> ::= <subscript 10><sign><digit> |
//                     <subscript 10><sign><digit><digit>
fn exponent_part<'a>() -> Parser<'a, String> {
    let sd_to_string = |(s, d): (String, char)| format!("{}{}", s, d);
    let sdd_to_string = |((s, d1), d2): ((String, char), char)| format!("{}{}{}", s, d1, d2);

    ((subscript10().discard() * sign() + digit()).map(sd_to_string)
        | (subscript10().discard() * sign() + digit() + digit()).map(sdd_to_string))
    .name("exponent_part")
}

// <sign> ::= <no character> | + | -
fn sign<'a>() -> Parser<'a, String> {
    (sym('+') | sym('-'))
        .repeat(..=1)
        .map(String::from_iter)
        .name("sign")
}

// <signed integer> ::= +<unsigned integer> | -<unsigned integer>
fn signed_integer<'a>() -> Parser<'a, IntType> {
    let negate = |i: IntType| -i;

    ((sym('+') * unsigned_integer()) | (sym('-') * unsigned_integer().map(negate)))
        .name("signed_integer")
}

// <unsigned integer> ::= <digit> | <digit><unsigned integer>
fn unsigned_integer<'a>() -> Parser<'a, IntType> {
    digit()
        .repeat(1..)
        .map(String::from_iter)
        .convert(|s| IntType::from_str(&s))
        .name("unsigned_integer")
}

// An accumulator address (an integer from 0 to 7).
// The accumulator address and terminating comma can be omitted
// if the user wishes to use accumulator 1.
fn acc<'a>() -> Parser<'a, Acc> {
    ((oct_dig() - sym(',')).map(Acc::from) | empty().map(|_| Acc::from(None))).name("acc")
}

// <absolute address> ::= <Unsigned integer>
fn absolute_address<'a>() -> Parser<'a, AddressRef> {
    unsigned_integer()
        .map(|i| i as AddressRef)
        .name("absolute_address")
}

fn mnemonic<'a>() -> Parser<'a, Mnemonic> {
    (exact("DBYTE").map(|_| Mnemonic::DBYTE)
        | exact("EXTRA").map(|_| Mnemonic::EXTRA)
        | exact("MOCKP").map(|_| Mnemonic::MOCKP)
        | exact("MOCKS").map(|_| Mnemonic::MOCKS)
        | exact("MPLYX").map(|_| Mnemonic::MULTX)
        | exact("MULTX").map(|_| Mnemonic::MULTX)
        | exact("NEQVX").map(|_| Mnemonic::NEQVX)
        | exact("SUBTX").map(|_| Mnemonic::SUBTX)
        | exact("ADDX").map(|_| Mnemonic::ADDX)
        | exact("ANDX").map(|_| Mnemonic::ANDX)
        | exact("DDIV").map(|_| Mnemonic::DDIV)
        | exact("DECR").map(|_| Mnemonic::DECR)
        | exact("DMULT").map(|_| Mnemonic::DMULT)
        | exact("DROT").map(|_| Mnemonic::DROT)
        | exact("DSHL").map(|_| Mnemonic::DSHL)
        | exact("DVDX").map(|_| Mnemonic::DVDX)
        | exact("EXEC").map(|_| Mnemonic::EXEC)
        | exact("INCR").map(|_| Mnemonic::INCR)
        | exact("JUMP").map(|_| Mnemonic::JUMP)
        | exact("MPLY").map(|_| Mnemonic::MULT)
        | exact("MULT").map(|_| Mnemonic::MULT)
        | exact("NEQV").map(|_| Mnemonic::NEQV)
        | exact("NTHG").map(|_| Mnemonic::NIL)
        | exact("PNEG").map(|_| Mnemonic::PNEG)
        | exact("PNOT").map(|_| Mnemonic::PNOT)
        | exact("POWR").map(|_| Mnemonic::POWR)
        | exact("PSQU").map(|_| Mnemonic::PSQU)
        | exact("PTYP").map(|_| Mnemonic::PTYP)
        | exact("PTYZ").map(|_| Mnemonic::PTYZ)
        | exact("PFFP").map(|_| Mnemonic::PFFP)
        | exact("SKAE").map(|_| Mnemonic::SKAE)
        | exact("SKAL").map(|_| Mnemonic::SKAL)
        | exact("SKAG").map(|_| Mnemonic::SKAG)
        | exact("SKAN").map(|_| Mnemonic::SKAN)
        | exact("SKED").map(|_| Mnemonic::SKED)
        | exact("SKEI").map(|_| Mnemonic::SKEI)
        | exact("SKET").map(|_| Mnemonic::SKET)
        | exact("SKIP").map(|_| Mnemonic::SKIP)
        | exact("SUBT").map(|_| Mnemonic::SUBT)
        | exact("NILX").map(|_| Mnemonic::NILX)
        | exact("SWAP").map(|_| Mnemonic::NILX)
        | exact("TAKE").map(|_| Mnemonic::TAKE)
        | exact("TNEG").map(|_| Mnemonic::TNEG)
        | exact("TNOT").map(|_| Mnemonic::TNOT)
        | exact("TOUT").map(|_| Mnemonic::TOUT)
        | exact("TSTR").map(|_| Mnemonic::TSTR)
        | exact("TTTT").map(|_| Mnemonic::TTTT)
        | exact("TTYP").map(|_| Mnemonic::TTYP)
        | exact("TTYZ").map(|_| Mnemonic::TTYZ)
        | exact("ADD").map(|_| Mnemonic::ADD)
        | exact("AND").map(|_| Mnemonic::AND)
        | exact("DIV").map(|_| Mnemonic::DIV)
        | exact("DVD").map(|_| Mnemonic::DVD)
        | exact("JAT").map(|_| Mnemonic::JAT)
        | exact("JEZ").map(|_| Mnemonic::JEZ)
        | exact("JGZ").map(|_| Mnemonic::JGZ)
        | exact("JLZ").map(|_| Mnemonic::JLZ)
        | exact("JNZ").map(|_| Mnemonic::JNZ)
        | exact("JZD").map(|_| Mnemonic::JZD)
        | exact("JZI").map(|_| Mnemonic::JZI)
        | exact("NIL").map(|_| Mnemonic::NIL)
        | exact("ORX").map(|_| Mnemonic::ORX)
        | exact("PIN").map(|_| Mnemonic::PIN)
        | exact("PUT").map(|_| Mnemonic::PUT)
        | exact("ROT").map(|_| Mnemonic::ROT)
        | exact("SHL").map(|_| Mnemonic::SHL)
        | exact("OR").map(|_| Mnemonic::OR)
        | exact("SQRT").map(|_| Mnemonic::SQRT)
        | exact("LN").map(|_| Mnemonic::LN)
        | exact("EXP").map(|_| Mnemonic::EXP)
        | exact("READ").map(|_| Mnemonic::READ)
        | exact("PRINT").map(|_| Mnemonic::PRINT)
        | exact("SIN").map(|_| Mnemonic::SIN)
        | exact("COS").map(|_| Mnemonic::COS)
        | exact("TAN").map(|_| Mnemonic::TAN)
        | exact("ATN").map(|_| Mnemonic::ATN)
        | exact("STOP").map(|_| Mnemonic::STOP)
        | exact("LINE").map(|_| Mnemonic::LINE)
        | exact("INT").map(|_| Mnemonic::INT)
        | exact("FRAC").map(|_| Mnemonic::FRAC)
        | exact("FLOAT").map(|_| Mnemonic::FLOAT)
        | exact("CAPTN").map(|_| Mnemonic::CAPTN)
        | exact("PAGE").map(|_| Mnemonic::PAGE)
        | exact("RND").map(|_| Mnemonic::RND)
        | exact("ABS").map(|_| Mnemonic::ABS))
    .name("mnemonic")
}

// Utility parsers
fn exact(tag: &str) -> Parser<'_, String> {
    let assert_tag = move |s| {
        (s == tag)
            .then_some(tag.into())
            .ok_or(Err::<String, _>("not tag"))
    };
    any()
        .repeat(tag.len())
        .map(String::from_iter)
        .convert(assert_tag)
        .name("exact")
}
