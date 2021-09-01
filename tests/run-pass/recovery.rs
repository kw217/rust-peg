extern crate peg;

// The sample grammar from the paper https://arxiv.org/abs/1806.11150
peg::parser!(grammar test_grammar() for str {
    rule _ = quiet!{ [' ' | '\r' | '\n' | '\t']* }
    rule name() = ['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*
    rule number() = ['0'..='9']+
    pub rule prog()
        = "public" _ "class" _ name() _ "{" _ "public" _ "static" _ "void" _ "main" _ "(" _ "String" _ "[" _ "]" _ name() _ ")" _ block_stmt() _ "}" _
    rule block_stmt() = "{" _ (stmt() _)* _ ("}" / error!("missing end of block" "xyzzy"))
    rule skip_to_rcur() = (!"}" ("{" skip_to_rcur() / [^ '}']))* "}"
    rule stmt() = if_stmt() / while_stmt() / print_stmt() / dec_stmt() / assign_stmt() / block_stmt()
    rule if_stmt() = "if" _ "(" _ exp() _ ")" _ stmt() _ ("else" _ stmt() _)?
    rule while_stmt() = "while" _ "(" _ exp() _ ")" _ stmt()
    rule dec_stmt() = "int" _ name() _ ( "=" _ exp() _)? ";"
    rule assign_stmt() = name() _ "=" _ exp() _ (";" / error!("missing semicolon in assignment" "xyzzy"))
    rule print_stmt() = "System.out.println" _ "(" _ exp() _ ")" _ ";"
    rule exp() = rel_exp() _ ("==" _ rel_exp() _)*
    rule rel_exp() = add_exp() _ ("<" _ add_exp() _)*
    rule add_exp() = mul_exp() _ (['+' | '-'] _ mul_exp() _)*
    rule mul_exp() = atom_exp() _ (['*' | '/'] _ atom_exp() _)*
    rule atom_exp() = "(" _ exp() _ ")" / number() / name()
});

use self::test_grammar::*;

fn main() {
    let input = concat!(
        "public class Example {\n",
        "  public static void main(String[] args) {\n",
        "    int n = 5;\n",
        "    int f = 1;\n",
        "    while(0 < n) {\n",
        "      f = f * n;\n",
        "      n = n - 1\n",
        "    };\n",
        "    System.out.println(f);\n",
        "  }\n",
        "}\n");
    let r = prog(input).unwrap();
    println!("{:?}", r);
    let err = prog(input).unwrap_err();

    // The best that can be done without recovery.
    assert_eq!(err.location.line, 8);
    assert_eq!(err.location.column, 5);
    assert_eq!(err.expected.to_string(), "\"}\"");
}
