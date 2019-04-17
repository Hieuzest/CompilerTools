
chars               [\ -~]
%

Symbol              <[\ -~^<>]*>

Assign              ::=
Alternation         \|

Literal?            "([\ -~^\\]|\\{chars})*"

SpecialSequence?    ?([\ -~^\\]|\\{chars})*?

-CommentLine        ;{chars}*
-CommentBlock?  	begin({chars}|\n|\t|\r|[a-zA-Z])end
NewLine             (\n|\r\n)*
-WhiteSpace         [\ \n\r\t]