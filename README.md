# Lox implementation in Rust

An implementation of Lox (for now only the interpreter, I will add the compiler later) in Rust. Since it's my very first interpreter, the logic follows closely the jlox implementation from the book [Crafting interpreters](http://craftinginterpreters.com/). 
However, I tried to write it in a more idiomatic Rust (e.g. using pattern matching instead of visitor pattern). 

Example

    class Person {
        init(name) {
            this.name = name;
        }

        hi() {
            print "Hi, I'm " + this.name + ".";
        }
    }

    class Employee < Person {
        init(name, profession) {
            super.init(name);
            this.profession = profession;
        }

        hi() {
            super.hi();
            print "I'm a " + this.profession + ".";
        }
    }

    var alice = Employee("Alice", "Software Engineer");
    alice.hi();


    fun fib(n) {
        if (n <= 1) {
            return 1;
        } else {
            return fib(n - 1) + fib(n - 2);
        }
    }

    for (var i = 0; i < 10; i = i + 1) {
        print fib(i);
    }
