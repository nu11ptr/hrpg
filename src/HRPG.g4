
// *** Parser rules ***

grammar HRPG;

topLevel
    : (entry NL)* entry?
    ;

entry
    : parseRule
    | tokenRule
    ;

parseRule
    : RULE_NAME NL_COLON ruleBody
    ;

ruleBody
    : rulePiece (NL_PIPE rulePiece)*
    ;

rulePiece
    : (RULE_NAME '=')? rulePart+
    ;

rulePart
    : ruleElem suffix?
    | LBRACKET ruleBody RBRACKET
    ;

ruleElem
    : '(' ruleBody ')'  # parensRuleBody
    | RULE_NAME         # tokRuleName
    | TOKEN_NAME        # tokTokenName
    | TOKEN_LIT         # tokTokenLit
    ;

suffix
    : PLUS          # tokPlus
    | STAR          # tokStar
    | QUEST_MARK    # tokQuestMark
    ;

tokenRule
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
