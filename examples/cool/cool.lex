
# This file describe the lexical of COOL language

chars           [\ -~]
digit           [0-9]
letterup        [A-Z]
letterlo        [a-z]
idletter        {letterup}+{letterlo}+{digit}+_
delimiter       [\(\)\{\}:;,]
operator        [\+\-\*/<=.]
newline         (\n|\r\n)

%

# Keyword

CLASS           [cC][lL][aA][sS][sS]
ELSE            [eE][lL][sS][eE]
# FALSE         f[aA][lL][sS][eE]
FI              [fF][iI]
IF              [iI][fF]
IN              [iI][nN]
INHERITS        [iI][nN][hH][eE][rR][iI][tT][sS]
ISVOID          [iI][sS][vV][oO][iI][dD]
LET             [lL][eE][tT]
LOOP            [lL][oO][oO][pP]
POOL            [pP][oO][oO][lL]
THEN            [tT][hH][eE][nN]
WHILE           [wW][hH][iI][lL][eE]
CASE            [cC][aA][sS][eE]
ESAC            [eE][sS][aA][cC]
NEW             [nN][eE][wW]
OF              [oO][fF]
NOT             [nN][oO][tT]
# TRUE          t[rR][uU][eE]


# General

BOOL_CONST      t[rR][uU][eE]|f[aA][lL][sS][eE]
OBJECTID        {letterlo}{idletter}*
TYPEID          {letterup}{idletter}*
INT_CONST       [1-9]{digit}*|0
STR_CONST?      "([\ -~^\\]|\\{chars})*"


# Operator

ASSIGN          <-
# OPERATOR      {operator}
OP_ADD          \+
OP_SUB          -
OP_MUL          \*
OP_DIV          /
OP_NEG          ~
OP_LT           <
OP_LE           <=
OP_EQ           =

# Delimiters

# DELIMITER     {delimiter}
DELIMITER       ;
COMMA           ,
LTUPLE          \(
RTUPLE          \)
LBLOCK          \{
RBLOCK          \}
DARROW          =>
TYPE_DEC        :
TYPE_ANN        @
DISPATCH        .

# Comment

-COMMENTLINE    \-\-({chars}|\t)*
-COMMENTBLOCK?  \(\*({chars}|\n|\t|\r)*\*\)
-WHITESPACE     [\ \n\r\t][\ \n\r\t]*