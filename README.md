# Atto

Atto is a ridiculously simple functional programming language for masochists.
It features a syntax driven entirely by polish notation and no delimiters to speak of.
If you make a mistake, your code will either be misinterpreted or will simply fail for no apparent reason.
What do you get for this simplicity? Well... an insanely simple self-evaluating language.

## Design

Atto's design is painfully simple. There are two kinds of structure:

Functions: `fn <name> [args] is <expr>`

Expressions: `<literal> [expr]`

That's it. Expressions, function calls, literals and operations are all considered to be the same thing.
Function signatures must appear before they are first used, otherwise Atto literally can't know how many arguments each function call has.
Despite this fact, Atto is, somehow, fully Turing-complete and it's actually possible - if a little annoying - to write perfectly functional (har har!) programs in it.

I leave you with a quick factorial calculation example demonstrating the compact expressiveness of Atto at work.

```
fn f is if = n 0 1 * n f - n 1
```

Yes, that's it.

## Atto Interpreter Written In Atto

In `examples/atto.at`, I've started working on an interpreter for Atto written in Atto.
At the moment, it's only capable of evaluating expressions: no function declarations are permitted.
In the future, I'd like to make it fully able to interpret Atto.

## Tutorial

Basic numeric operators:

```
fn main is
	+ 5 7

# Yields 12
```

```
fn main is
	- * 3 3 5

# Yields 4
```

Printing values to the console:

```
fn main is
	print "Hello, world!"
```

Receiving inputs from the user and converting them to a value:

```
fn main is
    print
    	* litr input "second: "
          litr input "first: "
```

Pairing values together into a two-component list:

```
fn main is
	pair 3 17

# Yields [3, 17]
```

Fusing lists together:

```
fn main is
	fuse pair 3 17 pair 5 8

# Yields [3, 17, 5, 8]
```

Conditional expression that returns the second operand if the first evaluates to true, and the third operand if not:

```
fn main is
	if true
		10
		5

# Yields 10
```

Selecting the first value in a list:

```
fn main is
	head pair 3 17

# Yields 3
```

Selecting values trailing after the head of a list:

```
fn main is
	tail fuse 3 fuse 17 9

# Yields [17, 9]
```

Converting a string into a value:

```
fn main is
	- 7 litr "3"

# Yields 4
```

```
fn main is
	= true litr "false"

# Yields false
```

```
fn main is
	= null litr "null"

# Yields true
```

Defining a function with parameters:

```
fn add x y is
	+ x y
fn main is
	add 5 3

# Yields 8
```

Recursion to find the size of a list:

```
fn size l is
	if = null head
		0
		+ 1 size tail l
fn main is
	size fuse 1 fuse 2 3

# Yields 3
```
