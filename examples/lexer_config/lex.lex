
newline         \n|\r\n

%


Delimiter       {newline}%       
Identifier      {newline}(\-|\0)[a-zA-Z_][a-zA-Z0-9_]*(\?|\0)[\ \t]
Alternation     \||\+

LCharGroup      \[
RCharGroup      \]
Alias           \{[a-zA-Z0-9_]*\}
LGroup          \(
RGroup          \)

LMatch          \(\?
RMatch          \?\)

CharRange       -
CharNeg         ^
Kleen           \*

Char            [\ -~^\ ] | \\[\ -~]

-Comment        {newline}#[\ -~\t]*
-NewLine        {newline}
-WhiteSpace     [\ \t][\ \t]*
