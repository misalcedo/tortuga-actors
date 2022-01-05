# Factorial

In this chapter we will implement [factorial](https://en.wikipedia.org/wiki/Factorial) using the Tortuga Programming Language. In mathematics, the `factorial` of a non-negative integer `n`, denoted by `n!`, is the product of all positive integers less than or equal to `n`. The factorial of `n` also equals the product of `n` with the next smaller factorial. The factorial of `0` is equal to `1` (i.e., `0! = 1`).

## Code
Create a file named `factorial.ta` with the following contents:

```tortuga
@integer(@n) = n - (n % 1)
@factorial(@n = 0) = 1
@factorial(@n > 0) = [
    @i = integer(n)
    i * factorial(i - 1)
]

factorial(9)
```

## Run
To run the file use the command-line interface from your favorite terminal:

```console
tortuga run factorial.ta
```

You should see the value `362880` printed to your terminal.
The Tortuga Programming Language automatically prints the value of the last expression in a program.