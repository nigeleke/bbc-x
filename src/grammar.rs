use crate::ast::*;

use pom::parser::*;

use std::str::FromStr;

// ****************************************************************************
// Pom representation of section 5 of
// https://spiral.imperial.ac.uk/bitstream/10044/1/21019/2/Tagg-W-1971-PhD-Thesis.pdf.
//
// "DEFINITION OF THE ASSEMBLER".
//
// The [BNF](https://en.wikipedia.org/wiki/Backus%E2%80%93Naur_form) style comments
// are from the thesis; Changes from that original are commented.

pub(crate) struct Grammar;

impl Grammar {
    pub fn bbc_x<'a>() -> Parser<'a, u8, SourceProgram> {
        source_program() - end()
    }
}

// ****************************************************************************
fn source_program<'a>() -> Parser<'a, u8, SourceProgram> {
    source_program_line()
        .repeat(1..)
        .name("source_program")
}

// ****************************************************************************
// <source program line> ::= <location><source program word>
//                           <comment><new line>
// Amended: Optional location; Optional whitespace.
fn source_program_line<'a>() -> Parser<'a, u8, SourceProgramLine> {
    (location().opt() +
        inline_ws().opt().discard() + 
        source_program_word() + 
        inline_ws().opt().discard() + 
        comment().opt() + 
        newline().discard())
        .map(|(((((l, _), w), _), c), _)| SourceProgramLine::new(l, w, c))
        .name("source_program_line")
}

// <location> ::= <numeric address><space>
// Amended: Location is symbolic and ends with ':'
fn location<'a>() -> Parser<'a, u8, Location> {
    (identifier() - colon()).name("location")
}

// <source program word> ::= <S-word> | <P-word> | <F-word> |
//                           <I-word> | <octal>
fn source_program_word<'a>() -> Parser<'a, u8, SourceProgramWord> {
    (   sword().map(SourceProgramWord::SWord) |
        pword().map(SourceProgramWord::PWord) |
        fword().map(SourceProgramWord::FWord) |
        iword().map(SourceProgramWord::IWord) |
        octal().map(SourceProgramWord::Octal)
    ).name("source_program_word")
}

// <comment> ::= <no character> | <space><string>
// Amended: Comment starts with ';'
fn comment<'a>() -> Parser<'a, u8, Comment> {
    sym(b';') * 
        none_of(b"\n")
            .repeat(0..)
            .convert(String::from_utf8)
            .name("comment")
}

// ****************************************************************************
// <S-word> ::= <quote><actual character><character><character>
//              <character><unquote>
// Amended: SWord delimited by new quote/unquote (", rather than "<" and ">").
fn sword<'a>() -> Parser<'a, u8, SWord> {
    (quote() + actual_character()
        .repeat(1..5) + unquote()) // TODO: Prefer inclusive range
        .convert(|((_, cs), _)| SWord::from_utf8(cs))
        .name("sword")
}

// <P-word> ::= <take type mnemonic><acc><general operand> |
//              <put type mnemonic>‹acc><address operand> |
//              LDN<acc><simple address operand>:<index>|
//              LDR<acc><const. operand>:‹index>|
//              LDR<acc><simple address operand>:<index>|
//              <library mnemonic>
fn pword<'a>() -> Parser<'a, u8, PWord> {
    (   take_type_pword() |
        put_type_pword() |
        loadn_pword() |
        loadr_const_pword() |
        loadr_pword() |
        library_mnemonic_pword()
    ).name("pword")
}

fn take_type_pword<'a>() -> Parser<'a, u8, PWord> {
    let as_take_type = |((((m, _), a), _), o)| PWord::TakeType(m, a, o);
    (take_type_mnemonic() +
        inline_ws().discard() + 
        acc() + 
        inline_ws().discard() + 
        general_operand()
    ).map(as_take_type)
     .name("take_type_pword")
}

fn put_type_pword<'a>() -> Parser<'a, u8, PWord> {
    let as_put_type = |((((m, _), a), _), o)| PWord::PutType(m, a, o);
    (put_type_mnemonic() +
        inline_ws().discard() +
        acc() + 
        inline_ws().discard() + 
        address_operand()
    ).map(as_put_type)
     .name("put_type_pword")
}

fn loadn_pword<'a>() -> Parser<'a, u8, PWord> {
    let as_loadn = |(((((_, _), a), _), o), i)| PWord::LoadN(a, o, i);
    (exact("LDN").discard() +
        inline_ws().discard() +
        acc() +
        inline_ws().discard() +
        simple_address_operand() + 
        index()
    ).map(as_loadn)
     .name("loadn_pword")
}

fn loadr_const_pword<'a>() -> Parser<'a, u8, PWord> {
    let as_loadr_const = |(((((_, _), a), _), o), i)| PWord::LoadRConst(a, o, i);
    (exact("LDR").discard() +
        inline_ws().discard() +
        acc() +
        inline_ws().discard() +
        const_operand() +
        index()
    ).map(as_loadr_const)
     .name("loadr_const_pword")
}

fn loadr_pword<'a>() -> Parser<'a, u8, PWord> {
    let as_loadr = |(((((_, _), a), _), o), i)| PWord::LoadR(a, o, i);
    (exact("LDR").discard() +
        inline_ws().discard() +
        acc() + 
        inline_ws().discard() + 
        simple_address_operand() + 
        index()
    ).map(as_loadr)
     .name("loadr_pword")
}

fn library_mnemonic_pword<'a>() -> Parser<'a, u8, PWord> {
    library_mnemonic()
        .map(PWord::LibraryMnemonic)
        .name("library_mnemonic_pword")
}

// <F-word> ::= <unsigned F-word> | <signed F-word>
fn fword<'a>() -> Parser<'a, u8, FWord> {
    (unsigned_fword() | signed_fword())
        .name("fword")
}

// <I-word> ::= <unsigned integer> | <signed integer>
fn iword<'a>() -> Parser<'a, u8, IWord> {
    (unsigned_integer() | signed_integer())
        .name("iword")
}

// <octal> ::= <type designator><oct.dig><oct.dig>‹oct.dig>
//             <oct.dig><oct.dig><oct.dig><oct.dig><oct.dig>
fn octal<'a>() -> Parser<'a, u8, Octal> {
    (type_designator() +
        oct_dig()
            .repeat(8)
            .convert(String::from_utf8)
            .convert(|s| WordValue::from_str_radix(&s, 8)))
        .map(|(t, value)|
            match t {
                b'S' => Octal::S(value),
                b'P' => Octal::P(value),
                b'F' => Octal::F(value),
                b'I' => Octal::I(value),
                _ => unreachable!(),
            }
        )
        .name("octal")
}

// <identifier> ::= <alpha char.> | <identifier><alpha char.> !
//                  <identifier><digit>
fn identifier<'a>() -> Parser<'a, u8, Identifier> {
    (alpha_character() + alpha_numeric()
        .repeat(0..))
        .convert(|(a, ans)| {
            let mut id = Vec::new();
            id.push(a);
            id.extend(ans);
            String::from_utf8(id)
        })
        .name("identifier")
}

fn colon<'a>() -> Parser<'a, u8, u8>  {
    sym(b':')
}

// Added: ++
fn alpha_numeric<'a>() -> Parser<'a, u8, u8>  {
    alpha_character() | digit()
} 
// Added: --

// <alpha character> ::= A | B | C | D | E | F | G | H | I | J | K| L |M | N | O | P |
//                       Q | R | S | T | V | W | X | Y | Z
fn alpha_character<'a>() -> Parser<'a, u8, u8>  {
    one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ")
}

// <oct.dig> ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
fn oct_dig<'a>() -> Parser<'a, u8, u8>  {
    one_of(b"01234567")
}

// <digit> ::= <oct.dig> | 8 | 9
fn digit<'a>() -> Parser<'a, u8, u8>  {
    oct_dig() | one_of(b"89")
}

// Added: ++
fn inline_ws<'a>() -> Parser<'a, u8, String> {
    one_of(b" \t")
        .repeat(1..)
        .convert(String::from_utf8)
}
// Added: --





// // // ****************************************************************************
// // // <string> ::= <character> | <character><string>
// // String = {
// //     Character,
// //     Character String
// // };

// <character> ::= <no character> | <actual character>
fn character<'a>() -> Parser<'a, u8, Option<Character>> {
    actual_character().opt()
}

// <actual character> ::= <alpha character> | <numeric character> |
//                        <punctuation>
fn actual_character<'a>() -> Parser<'a, u8, Character> {
    (alpha_character() | numeric_character())
        .name("actual_character") // TODO: | punctuation()
}

// <numeric character> ::= <digit> | + | - | <subscript 10> | .
fn numeric_character<'a>() -> Parser<'a, u8, NumericCharacter> {
    digit() | sym(b'+') | sym(b'-') | subscript10() | sym(b'.')
}

// // // <punctuation> ::= ( | <quote> | <unquote> | <apostrophe> | * | / |
// // //                   : | ) | = | ? | ^ | ~ | # | ; | , | <space>
// // // Amended: Use new Colon & SemiColon terminals. Unuse old Quote/Unquote.
// // Punctuation = {
// //     "(",
// //     "<",
// //     ">",
// //     Apostrophe,
// //     "*",
// //     "/",
// //     Colon,
// //     ")",
// //     "=",
// //     "?",
// //     "^",
// //     "~",
// //     "#",
// //     SemiColon,
// //     ",",
// //     Space,
// // };


// ****************************************************************************
// <no character>

// <quote>        .<.
// Amended: Redefined.
fn quote<'a>() -> Parser<'a, u8, u8>  {
    sym(b'"')
}

// <unquote>        .<.
// Amended: Redefined.
fn unquote<'a>() -> Parser<'a, u8, u8>  {
    sym(b'"')
}

// <apostrophe>   .'.
fn apostrophe<'a>() -> Parser<'a, u8, u8> {
    sym(b'\'')
}

// <space>        . .
fn space<'a>() -> Parser<'a, u8,u8> {
    sym(b' ')
}

// <new line>     .
// .
fn newline<'a>() -> Parser<'a, u8, u8> {
    sym(b'\n')
}

// <subscript 10> .@.
fn subscript10<'a>() -> Parser<'a, u8, u8>  {
    sym(b'@')
}

// // Added: ++
// Colon = ":";
// SemiColon = ";";
// // Added: --

// ****************************************************************************
// <general operand> ::= <address operand> | <const. operand>
fn general_operand<'a>() -> Parser<'a, u8, GeneralOperand> {
    (   address_operand().map(GeneralOperand::AddressOperand) |
        const_operand().map(GeneralOperand::ConstOperand)
    ).name("general_operand")
}

// <address operand> ::= <simple address operand> | <simple address operand>
//                       ":" <index>
// Amended: ':index' to '(index)'
fn address_operand<'a>() -> Parser<'a, u8, AddressOperand> {
    (simple_address_operand() + index_ref().opt())
        .map(|(o, i)| AddressOperand::new(o, i))
        .name("address_operand")
}

// <const. operand> ::= <signed integer> | <signed F-word> !
//                      <octal> | <S-word>
fn const_operand<'a>() -> Parser<'a, u8, ConstOperand> {
    (   signed_integer().map(ConstOperand::SignedInteger) |
        signed_fword().map(ConstOperand::SignedFWord) |
        octal().map(ConstOperand::Octal) |
        sword().map(ConstOperand::SWord)
    ).name("const_operand")    
}

// <simple address operand> ::= <space><address> | *<address>
fn simple_address_operand<'a>() -> Parser<'a, u8, SimpleAddressOperand> {
    (   (inline_ws() * address()).map(SimpleAddressOperand::DirectAddress) |
        (sym(b'*') * address()).map(SimpleAddressOperand::IndirectAddress)
    ).name("simple_address_operand")
}

// <address> ::= <identifier> | <numeric address>
fn address<'a>() -> Parser<'a, u8, Address> {
    (   identifier().map(Address::Identifier) |
        numeric_address().map(Address::NumericAddress)
    ).name("address")
}

// <numeric address> ::= <absolute address> | <relative address>
fn numeric_address<'a>() -> Parser<'a, u8, NumericAddress> {
    (   absolute_address().map(NumericAddress::AbsoluteAddress) |
        relative_address().map(NumericAddress::RelativeAddress)
    ).name("numeric_address")
}

// <absolute address> ::= <Unsigned integer>
fn absolute_address<'a>() -> Parser<'a, u8, AddressRef> {
    unsigned_integer()
        .map(|i| i as AddressRef)
        .name("absolute_address")
}

// <relative address> ::= <absolute address>+
// TODO: Implies forward reference only...
fn relative_address<'a>() -> Parser<'a, u8, RelativeRef> {
    (absolute_address() - sym(b'+'))
        .name("relative_address")
}

// <index> ::= <digit> | <digit><digit>
fn index<'a>() -> Parser<'a, u8, Index> {
    digit()
        .repeat(1..3) // TODO: Prefer 0..=2
        .convert(String::from_utf8)
        .convert(|s| Index::from_str(&s))
        .name("index")
}

fn index_ref<'a>() -> Parser<'a, u8, Index> {
    (sym(b'(') + index() + sym(b')'))
        .map(|((_, i), _)| i)
        .name("index_ref")
}

// <acc> ::= <no character> | 2
// Amended: Allow 0..7
fn acc<'a>() -> Parser<'a, u8, Acc> {
    oct_dig().opt().name("acc")
}

// ****************************************************************************
// <signed F-word> ::= +<unsigned F-word> | -<unsigned F-word>
fn signed_fword<'a>() -> Parser<'a, u8, FWord> {
    let negate = |f: FloatType| -f;

    (   (sym(b'+') * unsigned_fword()) |
        (sym(b'-') * unsigned_fword().map(negate))
    ).name("signed_fword")
}

// <unsigned F-word> ::= <decimal part> | <decimal part>
//                       <exponent part> | <unsigned integer>
//                       <exponent part>
fn unsigned_fword<'a>() -> Parser<'a, u8, FWord> {
    let de_to_float1 = |(d, e)| FloatType::from_str(&format!("{}e{}", d, e));
    let de_to_float2 = |(d, e)| FloatType::from_str(&format!("{}e{}", d, e));

    (   decimal_part().convert(|s| FloatType::from_str(&s)) |
        (decimal_part() + exponent_part()).convert(de_to_float1) |
        (unsigned_integer() + exponent_part()).convert(de_to_float2)
    ).name("unsigned_fword")
}

// <decimal part> ::= <unsigned integer>.<unsigned integer> |
//                    .<unsigned integer>
fn decimal_part<'a>() -> Parser<'a, u8, String> {
    let if_to_string = |(i, f): (IntType, IntType)| format!("{}.{}", i, f);
    let f_to_string = |f: IntType| format!("0.{}", f).into();

    (   (unsigned_integer() - sym(b'.').discard() + unsigned_integer()).map(if_to_string) |
        (sym(b'.').discard() * unsigned_integer()).map(f_to_string)
    ).name("decimal_part")
}

// <exponent part> ::= <subscript 10><sign><digit> |
//                     <subscript 10><sign><digit><digit>
fn exponent_part<'a>() -> Parser<'a, u8, String> {
    let sd_to_string = |(s, d): (String, u8)| format!("{}{}", s, d);
    let sdd_to_string = |((s, d1), d2): ((String, u8), u8)| format!("{}{}{}", s, d1, d2);

    (   (subscript10().discard() * sign() + digit()).map(sd_to_string) |
        (subscript10().discard() * sign() + digit() + digit()).map(sdd_to_string)
    ).name("exponent_part")
}

// <sign> ::= <no character> | + | -
fn sign<'a>() -> Parser<'a, u8, String> {
    (   sym(b'+') |
        sym(b'-')
    ).repeat(..=1)
     .convert(String::from_utf8)
     .name("sign")
}

// <signed integer> ::= +<unsigned integer> | -<unsigned integer>
fn signed_integer<'a>() -> Parser<'a, u8, IntType> {
    let negate = |i: IntType| -i;

    (   (sym(b'+') * unsigned_integer()) |
        (sym(b'-') * unsigned_integer().map(negate))
    ).name("signed_integer")
}

// <unsigned integer> ::= <digit> | <digit><unsigned integer>
fn unsigned_integer<'a>() -> Parser<'a, u8, IntType> {
    digit()
        .repeat(1..)
        .convert(String::from_utf8)
        .convert(|s| IntType::from_str(&s))
        .name("unsigned_integer")
}

// <type designator> ::= S | P | F | I
fn type_designator<'a>() -> Parser<'a, u8, TypeDesignator> {
    one_of(b"SPFI").name("type_designator")
}

// // // ****************************************************************************
// // // <mnemonic> ::= <take type mnemonic> | <put type mnemonic> |
// // //                LDN | LDR
// // Mnemonic = {
// //     TakeTypeMnemonic,
// //     PutTypeMnemonic,
// //     "LDN",
// //     "LDR",
// // };

// <take type mnemonic> ::= <0-15 mnemonic> | <skip mnemonic>
fn take_type_mnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    (n0_15_mnemonc() | skip_mnemonic()).name("take_type_mnemonic")
}

// <put type mnemonic> ::= X<0-15 mnemonic> | <16-22 mnemonic> |
//                         X<16-22 mnemonic> | <jump mnemonic>
fn put_type_mnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    (
        n0_15_xmnemonic() |
        n16_22_mnemonic() |
        n16_22_xmnemonic() |
        jump_mnemonic()
    ).name("put_type_mnemonic")
}

// <0-15 mnemonic> ::= NTHG | ADD | SUBT | MPLY | DVD | TAKE | NEG | MOD |
//                     CLR | AND | OR | NEQV | NOT | SHFR | CYCR | OPUT
fn n0_15_mnemonc<'a>() -> Parser<'a, u8, Mnemonic> {
    (   exact("NTHG").map(|_| Mnemonic::NTHG) |
        exact("ADD").map(|_| Mnemonic::ADD) |
        exact("SUBT").map(|_| Mnemonic::SUBT) |
        exact("MPLY").map(|_| Mnemonic::MPLY) |
        exact("DVD").map(|_| Mnemonic::DVD) |
        exact("TAKE").map(|_| Mnemonic::TAKE) |
        exact("NEG").map(|_| Mnemonic::NEG) |
        exact("MOD").map(|_| Mnemonic::MOD) |
        exact("CLR").map(|_| Mnemonic::CLR) |
        exact("AND").map(|_| Mnemonic::AND) |
        exact("OR").map(|_| Mnemonic::OR) |
        exact("NEQV").map(|_| Mnemonic::NEQV) |
        exact("NOT").map(|_| Mnemonic::NOT) |
        exact("SHFR").map(|_| Mnemonic::SHFR) |
        exact("CYCR").map(|_| Mnemonic::CYCR) |
        exact("OPUT").map(|_| Mnemonic::OPUT)
    ).name("n0_15_mnemonic")
}

fn n0_15_xmnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    let as_x = |m| match m {
        Mnemonic::NTHG => Mnemonic::XNTHG,
        Mnemonic::ADD => Mnemonic::XADD,
        Mnemonic::SUBT => Mnemonic::XSUBT,
        Mnemonic::MPLY => Mnemonic::XMPLY,
        Mnemonic::DVD => Mnemonic::XDVD,
        Mnemonic::TAKE => Mnemonic::XTAKE,
        Mnemonic::NEG => Mnemonic::XNEG,
        Mnemonic::MOD => Mnemonic::XMOD,
        Mnemonic::CLR => Mnemonic::XCLR,
        Mnemonic::AND => Mnemonic::XAND,
        Mnemonic::OR => Mnemonic::XOR,
        Mnemonic::NEQV => Mnemonic::XNEQV,
        Mnemonic::NOT => Mnemonic::XNOT,
        Mnemonic::SHFR => Mnemonic::XSHFR,
        Mnemonic::CYCR => Mnemonic::XCYCR,
        Mnemonic::OPUT => Mnemonic::XOPUT,
        _ => unreachable!(),
    };
    sym(b'X') * n0_15_mnemonc().map(as_x).name("n0_15_xmnemonic")
}

// <16-22 mnemonic> ::= IPUT | PUT | INCR | DECR | TYPE | CHYP | EXEC
fn n16_22_mnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    (   exact("IPUT").map(|_| Mnemonic::IPUT) |
        exact("PUT").map(|_| Mnemonic::PUT) |
        exact("INCR").map(|_| Mnemonic::INCR) |
        exact("DECR").map(|_| Mnemonic::DECR) |
        exact("TYPE").map(|_| Mnemonic::TYPE) |
        exact("CHYP").map(|_| Mnemonic::CHYP) |
        exact("EXEC").map(|_| Mnemonic::EXEC)
    ).name("n16_22_mnemonic")
}

fn n16_22_xmnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    let as_x = |m| match m {
        Mnemonic::IPUT => Mnemonic::XIPUT,
        Mnemonic::PUT => Mnemonic::XPUT,
        Mnemonic::INCR => Mnemonic::XINCR,
        Mnemonic::DECR => Mnemonic::XDECR,
        Mnemonic::TYPE => Mnemonic::XTYPE,
        Mnemonic::CHYP => Mnemonic::XCHYP,
        Mnemonic::EXEC => Mnemonic::XEXEC,
        _ => unreachable!(),
    };
    sym(b'X') * n16_22_mnemonic().map(as_x).name("n16_22_xmnemonic")
}

// <skip mnemonic>::= SKET | SKAE | SKAN | SKAL | SKAG
fn skip_mnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    (   exact("SKET").map(|_| Mnemonic::SKET) |
        exact("SKAE").map(|_| Mnemonic::SKAE) |
        exact("SKAN").map(|_| Mnemonic::SKAN) |
        exact("SKAL").map(|_| Mnemonic::SKAL) |
        exact("SKAG").map(|_| Mnemonic::SKAG)
    ).name("skip_mnemonic")
}

// <jump mnemonic> ::= LIBR | JLIK | JUMP | JEZ | JNZ | JLZ | JGZ | JOI |
//                     SLIK | SNLZ
fn jump_mnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    (   exact("LIBR").map(|_| Mnemonic::LIBR) |
        exact("JLIK").map(|_| Mnemonic::JLIK) |
        exact("JUMP").map(|_| Mnemonic::JUMP) |
        exact("JEZ").map(|_| Mnemonic::JEZ) |
        exact("JNZ").map(|_| Mnemonic::JNZ) |
        exact("JLZ").map(|_| Mnemonic::JLZ) |
        exact("JGZ").map(|_| Mnemonic::JGZ) |
        exact("JOI").map(|_| Mnemonic::JOI) |
        exact("SLIK").map(|_| Mnemonic::SLIK) |
        exact("SNLZ").map(|_| Mnemonic::SNLZ)
    ).name("jump_mnemonic")
}

// <library mnemonic> ::= SQRT | LN | EXP | READ | PRINT | SIN | COS | TAN |
//                        ARCTAN | STOP | LINE | INT | FRAC | FLOAT | CAPTN
fn library_mnemonic<'a>() -> Parser<'a, u8, Mnemonic> {
    (   exact("SQRT").map(|_| Mnemonic::SQRT) |
        exact("LN").map(|_| Mnemonic::LN) |
        exact("EXP").map(|_| Mnemonic::EXP) |
        exact("READ").map(|_| Mnemonic::READ) |
        exact("PRINT").map(|_| Mnemonic::PRINT) |
        exact("SIN").map(|_| Mnemonic::SIN) |
        exact("COS").map(|_| Mnemonic::COS) |
        exact("TAN").map(|_| Mnemonic::TAN) |
        exact("ARCTAN").map(|_| Mnemonic::ARCTAN) |
        exact("STOP").map(|_| Mnemonic::STOP) |
        exact("LINE").map(|_| Mnemonic::LINE) |
        exact("INT").map(|_| Mnemonic::INT) |
        exact("FRAC").map(|_| Mnemonic::FRAC) |
        exact("FLOAT").map(|_| Mnemonic::FLOAT) |
        exact("CAPTN").map(|_| Mnemonic::CAPTN)
    ).name("library_mnemonocic")
}

// Utility parsers
fn exact<'a>(tag: &'a str) -> Parser<'a, u8, String> {
    any()
        .repeat(tag.len())
        .convert(String::from_utf8)
        .convert(move |s| (s == tag).then_some(tag.into()).ok_or("error"))
        .name("exact")
}
