
// *** Parser rules ***

grammar HRPG;

top_level
    : (entry NL)* entry?
    ;

entry
    : parser_rule
    | token_rule
    ;

parser_rule
    : RULE_NAME NL_COLON rule_body
    ;

rule_body
    : rule_piece (NL_PIPE rule_piece)*
    ;

rule_piece
    : (RULE_NAME '=')? rule_part+
    ;

rule_part
    : rule_elem suffix?
    | LBRACKET rule_body RBRACKET
    ;

rule_elem
    : '(' rule_body ')' # parensRuleBody
    | RULE_NAME         # tokRuleName
    | TOKEN_NAME        # tokTokenName
    | TOKEN_LIT         # tokTokenLit
    ;

suffix
    : PLUS          # tokPlus
    | STAR          # tokStar
    | QUEST_MARK    # tokQuestMark
    ;

token_rule
    : TOKEN_NAME NL_COLON TOKEN_LIT
    ;

// *** Lexer rules ***

fragment BASE_NL: [\r]? [\n];
fragment BASE_WS: [ \t\f];

PLUS: '+';
STAR: '*';
QUEST_MARK: '?';
LBRACKET: '[';
RBRACKET: ']';

NL_COLON: NL? ':';
NL_PIPE: NL? '|';
NL: BASE_NL+ BASE_WS*;
TOKEN_NAME: [A-Z] [A-Z0-9_]*;
RULE_NAME: [a-z] [a-z0-9_]*;
TOKEN_LIT: '\'' .*? '\'';

WS: BASE_WS+ -> skip;
COMMENT: '#' ~[\r\n]* BASE_WS* -> skip;
