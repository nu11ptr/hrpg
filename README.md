# hrpg

Human Readable Parser Generator

## Overview

TLDR;

1. Human readable lexer parsers
2. Easily add language backends
3. (Almost) ANTLR4 compatible grammars

Writing a lexer/parser by hand, while not that difficult, can be extremely tedious. After the honeymoon period of the
new language has worn off, it can feel like downright drudgery at times. Instead, I propose a parser generator that
generates human-readable lexers and parsers that look like they might have been written by a human. This also allows the
freedom to use the parser generator as just a starting point and then take over by hand later.

A second goal is to support a wide number of language backends easily. To do this, we generate the lexer/parser into a
language agnostic AST. We then pass this AST into a base class that is designed to process this AST for typical
imperative style languages by only needing to define basic characteristics of the language. More exotic languages can be
generated, but they will take more customization work.

A third goal is to use a parser grammar format that is very similar to ANTLR4 to take advantage of the many available
grammars. The expected format is likely to be a bit more limited than ANTLR's, so some minor customization is likely due
to generated parser differences (precedence handling, etc.) and a slimmer feature set.

## High Level Operation

NOTE: Subject to change

1. **Frontend**: Lex/Parse Grammar --> Grammar AST
2. **Frontend**: Process/Optimize Grammar --> New Grammar AST
3. Generate Lexer/Parser --> Language Agnostic AST
4. **Backend**: Transform into Specific Language --> Language Specific AST
5. **Backend**: Write Lexer/Parser --> Lexer/Parser Language Files

## Status

In development - it is not ready for use in any way

## Building

### Fat Jar

The resulting JAR file can be found in `build/libs`. The 'fat' jar has an `-all` suffix.

```shell
./gradlew
```

### Native Image

The resulting `hrpg` binary can be found in `build/native/nativeCompile`

```shell
export GRAALVM_HOME=<graalvm_dir>
./gradlew nativeCompile
```
