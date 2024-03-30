use super::ast::*;

use pom::utf8::*;

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
    pub fn bbc3_line<'a>() -> Parser<'a, SourceProgramLine> {
        source_program_line()
    }
}

// ****************************************************************************
// <source program line> ::= <location><source program word>
//                           <comment><new line>
// Note: This implementation removes newlines.
fn source_program_line<'a>() -> Parser<'a, SourceProgramLine> {
    let as_source_program_line = |((l, w), c)| SourceProgramLine::new(l, w, c);

    (   location() +
        source_program_word() +
        comment() -
        end()
    ).map(as_source_program_line)
     .name("source_program_line")
}

// <location> ::= <numeric address><space>
// Amended: Changed to absolute address.
fn location<'a>() -> Parser<'a, Location> {
    (absolute_address() - inline_ws()).name("location")
}

// <source program word> ::= <S-word> | <P-word> | <F-word> |
//                           <I-word> | <octal>
fn source_program_word<'a>() -> Parser<'a, SourceProgramWord> {
    (   sword().map(SourceProgramWord::SWord) |
        pword().map(SourceProgramWord::PWord) |
        fword().map(SourceProgramWord::FWord) |
        iword().map(SourceProgramWord::IWord) |
        octal().map(SourceProgramWord::Octal)
    ).name("source_program_word")
}

// <comment> ::= <no character> | <space><string>
fn comment<'a>() -> Parser<'a, Comment> {
    let actual = || inline_ws() * 
        none_of("\n")
            .repeat(0..)
            .map(String::from_iter);
    (   actual() |
        empty().map(|_| "".to_string())
    )
     .name("comment")
}

// ****************************************************************************
// <S-word> ::= <quote><actual character><character><character>
//              <character><unquote>
fn sword<'a>() -> Parser<'a, SWord> {
    (   sym('<') +
        actual_character().repeat(1..=4) +
        sym('>')
    ).map(|((_, cs), _)| SWord::from_iter(cs.iter()))
     .name("sword")
}

// <P-word> ::= <take type mnemonic><acc><general operand> |
//              <put type mnemonic>‹acc><address operand> |
//              LDN<acc><simple address operand>:<index>|
//              LDR<acc><const. operand>:‹index>|
//              LDR<acc><simple address operand>:<index>|
//              <library mnemonic>
// Amended: Added optional whitespace between mnemonic, acc and operand.
fn pword<'a>() -> Parser<'a, PWord> {
    (   take_type_pword() |
        put_type_pword() |
        loadn_pword() |
        loadr_const_pword() |
        loadr_pword() |
        library_mnemonic_pword()
    ).name("pword")
}

fn take_type_pword<'a>() -> Parser<'a, PWord> {
    let as_take_type = |((m, a), o)| PWord::TakeType(m, a, o);

    (   take_type_mnemonic() -
        inline_ws().opt() +
        acc() -
        inline_ws().opt() +
        general_operand()
    ).map(as_take_type)
     .name("take_type_pword")
}

fn put_type_pword<'a>() -> Parser<'a, PWord> {
    let as_put_type = |((m, a), o)| PWord::PutType(m, a, o);

    (   put_type_mnemonic() -
        inline_ws().opt() +
        acc() -
        inline_ws().opt() +
        address_operand()
    ).map(as_put_type)
     .name("put_type_pword")
}

fn loadn_pword<'a>() -> Parser<'a, PWord> {
    let as_loadn = |((a, o), i)| PWord::LoadN(a, o, i);

    (   ldn_mnemonic() *
        inline_ws().opt() *
        acc() -
        inline_ws().opt() +
        simple_address_operand() + 
        index_ref()
    ).map(as_loadn)
     .name("loadn_pword")
}

fn loadr_const_pword<'a>() -> Parser<'a, PWord> {
    let as_loadr_const = |((a, o), i)| PWord::LoadRConst(a, o, i);

    (   ldr_mnemonic() *
        inline_ws().opt() *
        acc() -
        inline_ws().opt() +
        const_operand() +
        index_ref()
    ).map(as_loadr_const)
     .name("loadr_const_pword")
}

fn loadr_pword<'a>() -> Parser<'a, PWord> {
    let as_loadr = |((a, o), i)| PWord::LoadR(a, o, i);

    (   ldr_mnemonic() *
        inline_ws().opt() *
        acc() -
        inline_ws().opt() +
        simple_address_operand() + 
        index_ref()
    ).map(as_loadr)
     .name("loadr_pword")
}

fn library_mnemonic_pword<'a>() -> Parser<'a, PWord> {
    library_mnemonic()
        .map(PWord::LibraryMnemonic)
        .name("library_mnemonic_pword")
}

// <F-word> ::= <unsigned F-word> | <signed F-word>
fn fword<'a>() -> Parser<'a, FWord> {
    (unsigned_fword() | signed_fword())
        .name("fword")
}

// <I-word> ::= <unsigned integer> | <signed integer>
fn iword<'a>() -> Parser<'a, IWord> {
    (unsigned_integer() | signed_integer())
        .name("iword")
}

// <octal> ::= <type designator><oct.dig><oct.dig>‹oct.dig>
//             <oct.dig><oct.dig><oct.dig><oct.dig><oct.dig>
fn octal<'a>() -> Parser<'a, Octal> {
    let as_enum = |(t, value)| match t {
        'S' => Octal::S(value),
        'P' => Octal::P(value),
        'F' => Octal::F(value),
        'I' => Octal::I(value),
        _ => unreachable!(),
    };

    (   sym('(') *
        type_designator() +
        octal_word () -
        sym(')')
    ).map(as_enum)
     .name("octal")
}

fn octal_word<'a>() -> Parser<'a, WordValue> {
    oct_dig()
        .repeat(8)
        .map(String::from_iter)
        .convert(|s| WordValue::from_str_radix(&s, 8))
        .name("octal_word")
}

// ****************************************************************************
// <string> ::= <character> | <character><string>
// Amended: Definition not required.

// <character> ::= <no character> | <actual character>
// Amended: Definition not required.

// <actual character> ::= <alpha character> | <numeric character> |
//                        <punctuation>
fn actual_character<'a>() -> Parser<'a, Character> {
    (   alpha_character() | 
        numeric_character() |
        punctuation()
    ).name("actual_character")
}

// <alpha character> ::= A | B | C | D | E | F | G | H | I | J | K| L |M | N | O | P |
//                       Q | R | S | T | V | W | X | Y | Z
fn alpha_character<'a>() -> Parser<'a, char>  {
    one_of(&('A'..='Z'))
}

// <numeric character> ::= <digit> | + | - | <subscript 10> | .
fn numeric_character<'a>() -> Parser<'a, NumericCharacter> {
    (   digit() | 
        sym('+') |
        sym('-') |
        subscript10() |
        sym('.')
    ).name("numeric_character")
}

// <punctuation> ::= ( | <quote> | <unquote> | <apostrophe> | * | / |
//                   : | ) | = | ? | ^ | ~ | # | ; | , | <space>
fn punctuation<'a>() -> Parser<'a, Punctuation> {
    one_of("<>'*/:)=?^~#;. ").name("punctuation")
}

// <digit> ::= <oct.dig> | 8 | 9
fn digit<'a>() -> Parser<'a, char>  {
    oct_dig() | one_of("89")
}

// <oct.dig> ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
fn oct_dig<'a>() -> Parser<'a, char>  {
    one_of(&('0'..='7'))
}

// Added: ++
// Represents 'space()' in descriptipn
fn inline_ws<'a>() -> Parser<'a, String> {
    one_of(" \t")
        .repeat(1..)
        .map(String::from_iter)
        .name("inline_ws")
}
// Added: --


// ****************************************************************************
// Amended: Most definitions not directly required....
// <no character>
// <quote>        .<.
// <unquote>        .<.
// <apostrophe>   .'.
// <space>        . .
// <new line>     .
// .
// <subscript 10> .@.
fn subscript10<'a>() -> Parser<'a, char>  { sym('@') }

// ****************************************************************************
// <general operand> ::= <address operand> | <const. operand>
fn general_operand<'a>() -> Parser<'a, GeneralOperand> {
    (   const_operand().map(GeneralOperand::ConstOperand) |
        address_operand().map(GeneralOperand::AddressOperand)
    ).name("general_operand")
}

// <address operand> ::= <simple address operand> | <simple address operand>
//                       ":" <index>
fn address_operand<'a>() -> Parser<'a, AddressOperand> {
    (   simple_address_operand() +
        index_ref().opt()
    ).map(|(o, i)| AddressOperand::new(o, i))
     .name("address_operand")
}

// <const. operand> ::= <signed integer> | <signed F-word> !
//                      <octal> | <S-word>
fn const_operand<'a>() -> Parser<'a, ConstOperand> {
    inline_ws().opt() *
    (   signed_fword().map(ConstOperand::SignedFWord) |
        signed_integer().map(ConstOperand::SignedInteger) |
        octal().map(ConstOperand::Octal) |
        sword().map(ConstOperand::SWord)
    ).name("const_operand")    
}

// <simple address operand> ::= <space><address> | *<address>
// Amended: <space> handled elsewhere.
fn simple_address_operand<'a>() -> Parser<'a, SimpleAddressOperand> {
    (   address().map(SimpleAddressOperand::DirectAddress) |
        (sym('*') * address()).map(SimpleAddressOperand::IndirectAddress)
    ).name("simple_address_operand")
}

// <address> ::= <identifier> | <numeric address>
fn address<'a>() -> Parser<'a, Address> {
    (   identifier().map(Address::Identifier) |
        numeric_address().map(Address::NumericAddress)
    ).name("address")
}

// <identifier> ::= <alpha char.> | <identifier><alpha char.> !
//                  <identifier><digit>
fn identifier<'a>() -> Parser<'a, Identifier> {
    let concat = |a: char, ans: &[char]| {
        let mut id = Vec::from(ans);
        id.insert(0, a);
        String::from_iter(id)
    }; 

    (   alpha_character() + 
        alpha_numeric().repeat(0..)
    ).map(move |ans| concat(ans.0, &ans.1))
     .name("identifier")
}

fn alpha_numeric<'a>() -> Parser<'a, char>  {
    alpha_character() | digit()
} 

// <numeric address> ::= <absolute address> | <relative address>
fn numeric_address<'a>() -> Parser<'a, NumericAddress> {
    (   relative_address().map(NumericAddress::RelativeAddress) |
        absolute_address().map(NumericAddress::AbsoluteAddress)
    ).name("numeric_address")
}

// <absolute address> ::= <Unsigned integer>
fn absolute_address<'a>() -> Parser<'a, AddressRef> {
    unsigned_integer()
        .map(|i| i as AddressRef)
        .name("absolute_address")
}

// <relative address> ::= <absolute address>+
fn relative_address<'a>() -> Parser<'a, RelativeRef> {
    (   absolute_address() - 
        sym('+')
    ).name("relative_address")
}

// <index> ::= <digit> | <digit><digit>
fn index<'a>() -> Parser<'a, Index> {
    digit()
        .repeat(1..=2)
        .map(String::from_iter)
        .convert(|s| Index::from_str(&s))
        .name("index")
}

fn index_ref<'a>() -> Parser<'a, Index> {
    (sym(':') * index())
        .map(|i| i)
        .name("index_ref")
}

// <acc> ::= <no character> | 2
fn acc<'a>() -> Parser<'a, Acc> {
    (   sym('2').map(Acc::from) |
        empty().map(|_| Acc::from(None))
    ).name("acc")
}

// ****************************************************************************
// <signed F-word> ::= +<unsigned F-word> | -<unsigned F-word>
fn signed_fword<'a>() -> Parser<'a, FWord> {
    let negate = |f: FloatType| -f;

    (   (sym('+') * unsigned_fword()) |
        (sym('-') * unsigned_fword().map(negate))
    ).name("signed_fword")
}

// <unsigned F-word> ::= <decimal part> | <decimal part>
//                       <exponent part> | <unsigned integer>
//                       <exponent part>
fn unsigned_fword<'a>() -> Parser<'a, FWord> {
    let de_to_float1 = |(d, e)| FloatType::from_str(&format!("{}e{}", d, e));
    let de_to_float2 = |(d, e)| FloatType::from_str(&format!("{}e{}", d, e));

    (   decimal_part().convert(|s| FloatType::from_str(&s)) |
        (decimal_part() + exponent_part()).convert(de_to_float1) |
        (unsigned_integer() + exponent_part()).convert(de_to_float2)
    ).name("unsigned_fword")
}

// <decimal part> ::= <unsigned integer>.<unsigned integer> |
//                    .<unsigned integer>
fn decimal_part<'a>() -> Parser<'a, String> {
    let if_to_string = |(i, f): (IntType, IntType)| format!("{}.{}", i, f);
    let f_to_string = |f: IntType| format!("0.{}", f);

    (   (unsigned_integer() - sym('.').discard() + unsigned_integer()).map(if_to_string) |
        (sym('.').discard() * unsigned_integer()).map(f_to_string)
    ).name("decimal_part")
}

// <exponent part> ::= <subscript 10><sign><digit> |
//                     <subscript 10><sign><digit><digit>
fn exponent_part<'a>() -> Parser<'a, String> {
    let sd_to_string = |(s, d): (String, char)| format!("{}{}", s, d);
    let sdd_to_string = |((s, d1), d2): ((String, char), char)| format!("{}{}{}", s, d1, d2);

    (   (subscript10().discard() * sign() + digit()).map(sd_to_string) |
        (subscript10().discard() * sign() + digit() + digit()).map(sdd_to_string)
    ).name("exponent_part")
}

// <sign> ::= <no character> | + | -
fn sign<'a>() -> Parser<'a, String> {
    (   sym('+') |
        sym('-')
    ).repeat(..=1)
     .map(String::from_iter)
     .name("sign")
}

// <signed integer> ::= +<unsigned integer> | -<unsigned integer>
fn signed_integer<'a>() -> Parser<'a, IntType> {
    let negate = |i: IntType| -i;

    (   (sym('+') * unsigned_integer()) |
        (sym('-') * unsigned_integer().map(negate))
    ).name("signed_integer")
}

// <unsigned integer> ::= <digit> | <digit><unsigned integer>
fn unsigned_integer<'a>() -> Parser<'a, IntType> {
    digit()
        .repeat(1..)
        .map(String::from_iter)
        .convert(|s| IntType::from_str(&s))
        .name("unsigned_integer")
}

// <type designator> ::= S | P | F | I
fn type_designator<'a>() -> Parser<'a, TypeDesignator> {
    one_of("SPFI").name("type_designator")
}

// ****************************************************************************
// <mnemonic> ::= <take type mnemonic> | <put type mnemonic> |
//                LDN | LDR
// Not required in grammar...

// <take type mnemonic> ::= <0-15 mnemonic> | <skip mnemonic>
fn take_type_mnemonic<'a>() -> Parser<'a, Mnemonic> {
    (   n0_15_mnemonc() |
        skip_mnemonic()
    ).name("take_type_mnemonic")
}

// <put type mnemonic> ::= X<0-15 mnemonic> | <16-22 mnemonic> |
//                         X<16-22 mnemonic> | <jump mnemonic>
fn put_type_mnemonic<'a>() -> Parser<'a, Mnemonic> {
    (   n0_15_xmnemonic() |
        n16_22_mnemonic() |
        n16_22_xmnemonic() |
        jump_mnemonic()
    ).name("put_type_mnemonic")
}

// <0-15 mnemonic> ::= NTHG | ADD | SUBT | MPLY | DVD | TAKE | NEG | MOD |
//                     CLR | AND | OR | NEQV | NOT | SHFR | CYCR | OPUT
fn n0_15_mnemonc<'a>() -> Parser<'a, Mnemonic> {
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

fn n0_15_xmnemonic<'a>() -> Parser<'a, Mnemonic> {
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

    (   sym('X') *
        n0_15_mnemonc().map(as_x)
    ).name("n0_15_xmnemonic")
}

// <16-22 mnemonic> ::= IPUT | PUT | INCR | DECR | TYPE | CHYP | EXEC
fn n16_22_mnemonic<'a>() -> Parser<'a, Mnemonic> {
    (   exact("IPUT").map(|_| Mnemonic::IPUT) |
        exact("PUT").map(|_| Mnemonic::PUT) |
        exact("INCR").map(|_| Mnemonic::INCR) |
        exact("DECR").map(|_| Mnemonic::DECR) |
        exact("TYPE").map(|_| Mnemonic::TYPE) |
        exact("CHYP").map(|_| Mnemonic::CHYP) |
        exact("EXEC").map(|_| Mnemonic::EXEC)
    ).name("n16_22_mnemonic")
}

fn n16_22_xmnemonic<'a>() -> Parser<'a, Mnemonic> {
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
    sym('X') * n16_22_mnemonic().map(as_x).name("n16_22_xmnemonic")
}

// <skip mnemonic>::= SKET | SKAE | SKAN | SKAL | SKAG
fn skip_mnemonic<'a>() -> Parser<'a, Mnemonic> {
    (   exact("SKET").map(|_| Mnemonic::SKET) |
        exact("SKAE").map(|_| Mnemonic::SKAE) |
        exact("SKAN").map(|_| Mnemonic::SKAN) |
        exact("SKAL").map(|_| Mnemonic::SKAL) |
        exact("SKAG").map(|_| Mnemonic::SKAG)
    ).name("skip_mnemonic")
}

// <jump mnemonic> ::= LIBR | JLIK | JUMP | JEZ | JNZ | JLZ | JGZ | JOI |
//                     SLIK | SNLZ
fn jump_mnemonic<'a>() -> Parser<'a, Mnemonic> {
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
fn library_mnemonic<'a>() -> Parser<'a, Mnemonic> {
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

fn ldn_mnemonic<'a>() -> Parser<'a, Mnemonic> {
    exact("LDN")
        .map(|_| Mnemonic::LDN)
        .name("ldn_mnemonic")
}

fn ldr_mnemonic<'a>() -> Parser<'a, Mnemonic> {
    exact("LDR")
        .map(|_| Mnemonic::LDR)
        .name("ldr_mnemonic")
}

// Utility parsers
fn exact(tag: &str) -> Parser<'_, String> {
    let assert_tag = move |s| (s == tag).then_some(tag.into()).ok_or(Err::<String, _>("not tag"));
    any()
        .repeat(tag.len())
        .map(String::from_iter)
        .convert(assert_tag)
        .name("exact")
}
