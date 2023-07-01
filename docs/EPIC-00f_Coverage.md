# Code Coverage

## The Gold Standard

There is a state in any codebase that I like to call the Gold Standard. It's very hard to achieve but when you do give
yourself and the people you work with. Here it is:

> **_Any time you change how your code interacts with any other parts of the code it should make a test that passed in
> the past fail._**

Now I am not saying that you have to always meet this standard, just to watch for it. If you suddenly do a significant
change to the functionality of your code and no test fails, try to figure out why.
