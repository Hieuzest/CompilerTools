
# This file describe the lexical of EBNF used in the parser
# The EBNF is used to describe the syntax of language

# chars			    [0-9a-zA-Z_\+\-\*/=<>.@~!,\{\}\(\);'\ :\\"\[\]\|]
chars               [\ -~]
%

ProductionName      [a-zA-Z_][a-zA-Z0-9_]*

LeftPrecedence      \|[0-9]*>
RightPrecedence     \|[0-9]*<

Assign              =
Terminator          .
Alternation         \|
TokenRange          ...
TokenValue          <\-

Token?              "([\ -~^\\]|\\{chars})*"


LeftOptional        \[
RightOptional       \]

LeftGroup    		\(
RightGroup          \)

LeftRepetition      \{
RightRepetition     \}

LeftUnwrap          <
RightUnwrap         >

SpecialSequence?    ?([\ -~^\\]|\\{chars})*?

-CommentBlock?  	\(\*({chars}|\n|\t|\r|[a-zA-Z])*\*\)
-WhiteSpace         [\ \n\r\t]