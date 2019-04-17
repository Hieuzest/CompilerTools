
newline                 (\n|\r\n)
empty                   \0
chars                   [\ -~]

whitespace              [\ \n\r\t]
delimiter               {whitespace}|[\(\)";]
letter                  [a-zA-Z]
special_initial         [!$%&\*/:<=>?_^]
digit                   [0-9]
special_subsequent      [\+\-.@]

initial                 {letter}|{special_initial}
peculiar_identifier     [\+\-]|...
subsequent              {initial}|{digit}|{special_subsequent}

identifier              {initial}{subsequent}* | {peculiar_identifier}

comment                 ;{subsequent}*
atmosphere              {whitespace}
intertoken_space        {atmosphere}*

exponent_marker         e|s|f|d|l
sign                    \0|\+|\-
exactness               \0|#i|#e

digit_2                 [01]
digit_8                 [0-7]
digit_10                {digit}
digit_16                {digit}|[a-fA-F]
radix_2                 #b
radix_8                 #o
radix_10                \0|#d
radix_16                #x

suffix                  {empty}|{exponent_marker}{sign}{digit_10}{digit_10}*

prefix_2                {radix_2}{exactness}|{exactness}{radix_2}
prefix_8                {radix_8}{exactness}|{exactness}{radix_8}
prefix_10               {radix_10}{exactness}|{exactness}{radix_10}
prefix_16               {radix_16}{exactness}|{exactness}{radix_16}

uinteger_2              {digit_2}{digit_2}*#*
uinteger_8              {digit_8}{digit_8}*#*
uinteger_10             {digit_10}{digit_10}*#*
uinteger_16             {digit_16}{digit_16}*#*

decimal_10              {uinteger_10}{suffix}|.{digit_10}{digit_10}*#*{suffix}|{digit_10}{digit_10}*.{digit_10}*#*{suffix}|{digit_10}{digit_10}*##*.#*{suffix}

ureal_2                 {uinteger_2}|{uinteger_2}/{uinteger_2}
ureal_8                 {uinteger_8}|{uinteger_8}/{uinteger_8}
ureal_10                {uinteger_10}|{uinteger_10}/{uinteger_10}|{decimal_10}
ureal_16                {uinteger_16}|{uinteger_16}/{uinteger_16}

real_2                  {sign}{ureal_2}
real_8                  {sign}{ureal_8}
real_10                 {sign}{ureal_10}
real_16                 {sign}{ureal_16}

complex_2               {real_2} | {real_2}@{real_2} | {real_2}\+{ureal_2}i | {real_2}\-{ureal_2}i | {real_2}\+i | {real_2}\-i | \+{ureal_2}i | \-{ureal_2}i | \+i | \-i
complex_8               {real_8} | {real_8}@{real_8} | {real_8}\+{ureal_8}i | {real_8}\-{ureal_8}i | {real_8}\+i | {real_8}\-i | \+{ureal_8}i | \-{ureal_8}i | \+i | \-i
complex_10              {real_10} | {real_10}@{real_10} | {real_10}\+{ureal_10}i | {real_10}\-{ureal_10}i | {real_10}\+i | {real_10}\-i | \+{ureal_10}i | \-{ureal_10}i | \+i | \-i
complex_16              {real_16} | {real_16}@{real_16} | {real_16}\+{ureal_16}i | {real_16}\-{ureal_16}i | {real_16}\+i | {real_16}\-i | \+{ureal_16}i | \-{ureal_16}i | \+i | \-i

num_2                   {prefix_2}{complex_2}
num_8                   {prefix_8}{complex_8}
num_10                  {prefix_10}{complex_10}
num_16                  {prefix_16}{complex_16}

character_name          space|newline
string_element          [!#-~\ ]

%

# Keywords

Else                    else
Darrow                  =>
Define                  define
Unquote                 unquote
Unquote_splicing        unquote-splicing

Eval                    eval
Apply                   apply

Quote                   quote
Lambda                  lambda
If                      if
Set                     set!
Begin                   begin
Cond                    cond
And                     and
Or                      or
Case                    case
Let                     let
Letstar                 let\*
Letrec                  letrec
Do                      do
Delay                   delay
Quasiquote              quasiquote

DefineSyntax            define-syntax

# Operators

Symbolize               '
Template                `
Comma                   ,
Comma_Splicing          ,@
# Macro                   !
Dot                     .
# Cast                    \->

LGroup                  \(
RGroup                  \)
VGroup                  #\(


Identifier              {identifier}

# Literals

Number                  {num_2}|{num_8}|{num_10}|{num_16}
Character               #\\[!-~]|#\\{character_name}
String                  "{string_element}*"
Boolean                 #t|#f



-Comment                ;({chars}|\t|\ |;)*
-Intertoken_Space       {intertoken_space}
