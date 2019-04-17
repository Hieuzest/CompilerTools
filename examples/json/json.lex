chars               [\ -~]
%

Comma               ,
Colon               :

Int                 (-|\0)0|[1-9][0-9]*
Double              (-|\0)0|[1-9][0-9]*.[0-9][0-9]*
True                true
False               false
Null                null
String?             "([\ -~^\\]|\\{chars})*"

LeftArray           \[
RightArray          \]

LeftObject          \{
RightObject         \}

-WhiteSpace         [\ \n\r\t]