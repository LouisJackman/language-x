package lexing;

import java.math.BigDecimal;
import java.util.*;
import java.util.function.BooleanSupplier;
import java.util.function.Function;
import java.util.function.Predicate;

import static java.lang.String.format;
import static java.util.Objects.requireNonNull;

/**
 * Lexes a source file into a list of tokens.
 */
public final class Lexer implements Iterator<Token<?>> {
    private final static Map<Keyword, Token.Symbolic> keywords;

    static {
        Map<Keyword, Token.Symbolic> mutableKeywords = new HashMap<>();
        mutableKeywords.put(Keyword.ABSTRACT, new Token.Abstract());
        mutableKeywords.put(Keyword.ACTOR, new Token.Abstract());
        mutableKeywords.put(Keyword.CASE, new Token.Case());
        mutableKeywords.put(Keyword.CLASS, new Token.Class());
        mutableKeywords.put(Keyword.CONTINUE, new Token.Continue());
        mutableKeywords.put(Keyword.DEFAULT, new Token.Default());
        mutableKeywords.put(Keyword.DO, new Token.Do());
        mutableKeywords.put(Keyword.ELSE, new Token.Else());
        mutableKeywords.put(Keyword.EXTENDS, new Token.Extends());
        mutableKeywords.put(Keyword.FOR, new Token.For());
        mutableKeywords.put(Keyword.GET, new Token.Get());
        mutableKeywords.put(Keyword.IF, new Token.If());
        mutableKeywords.put(Keyword.IMPLEMENTS, new Token.Implements());
        mutableKeywords.put(Keyword.IMPORT, new Token.Import());
        mutableKeywords.put(Keyword.INTERFACE, new Token.Interface());
        mutableKeywords.put(Keyword.INTERNAL, new Token.Internal());
        mutableKeywords.put(Keyword.OVERRIDE, new Token.OverrideToken());
        mutableKeywords.put(Keyword.PACKAGE, new Token.Package());
        mutableKeywords.put(Keyword.PUBLIC, new Token.Public());
        mutableKeywords.put(Keyword.RECEIVE, new Token.Receive());
        mutableKeywords.put(Keyword.SUPER, new Token.Super());
        mutableKeywords.put(Keyword.SWITCH, new Token.Switch());
        mutableKeywords.put(Keyword.THROW, new Token.Throw());
        mutableKeywords.put(Keyword.TIMEOUT, new Token.Timeout());
        mutableKeywords.put(Keyword.UNDERSCORE, new Token.Ignore());
        mutableKeywords.put(Keyword.VAR, new Token.Var());
        keywords = Collections.unmodifiableMap(mutableKeywords);
    }

    private final SourceFileStream input;

    public Lexer(SourceFileStream input) {
        this.input = requireNonNull(input);
    }

    @Override
    public boolean hasNext() {
        skipWhitespace();
        return !input.isEmpty();
    }

    @Override
    public Token<?> next() {
        skipWhitespace();
        return readPending().orElseThrow(NoSuchElementException::new);
    }

    private void skipWhitespace() {
        input.lookAhead().ifPresent(c -> {
            if (Character.isWhitespace(c)) {
                input.read();
                skipWhitespace();
            }
        });
    }

    private Optional<Token<?>> readPending() {
        Function<Predicate<Character>, BooleanSupplier> secondCharIs = predicate -> () -> input
                .lookAhead(2)
                .map(s -> s.charAt(1))
                .filter(predicate)
                .isPresent();

        BooleanSupplier secondCharIsDigit = secondCharIs.apply(Character::isDigit);
        BooleanSupplier secondCharIsAstrix = secondCharIs.apply(c -> c == '*');
        BooleanSupplier secondCharIsForwardSlash = secondCharIs.apply(c -> c == '/');

        return input.lookAhead().flatMap(c -> {
            final Optional<Token<?>> result;
            if ((c == 'v') && secondCharIsDigit.getAsBoolean()) {
                result = lexVersion();
            } else if (Character.isLetter(c) || (c == '_')) {
                input.read();
                result = Optional.of(lexBooleanOrKeywordOrIdentifier(c + lexRestOfWord()));
            } else if (c == '"') {
                result = lexString();
            } else if (c == '$') {
                result = lexInterpolatedString();
            } else if (c == '\'') {
                result = lexChar();
            } else if (
                    Character.isDigit(c)
                            || ((c == '+') && (secondCharIsDigit.getAsBoolean()))
                            || ((c == '-') && (secondCharIsDigit.getAsBoolean()))
                    ) {
                result = lexNumber();
            } else if ((c == '/') && secondCharIsAstrix.getAsBoolean()) {
                result = lexMultiLineComment();
            } else if ((c == '/') && secondCharIsForwardSlash.getAsBoolean()) {
                result = Optional.of(lexSingleLineComment());
            } else {
                result = lexOperator();
            }
            return result;
        });
    }

    private Optional<Token<?>> lexVersion() {
        input.read();
        return lexAbsoluteNumber().map(Token.Version::new);
    }

    private String lexRestOfWord() {
        return input
                .lookAhead()
                .map(c -> {
                    final String result;
                    if (Character.isLetter(c) || Character.isDigit(c)) {
                        input.read();
                        result = c + lexRestOfWord();
                    } else {
                        result = "";
                    }
                    return result;
                })
                .orElse("");
    }

    private Optional<Token<?>> lexString() {
        input.read();
        return lexStringGen().map(Token.StringToken::new);
    }

    private Optional<String> lexStringGen() {
        return input.read().flatMap(c -> {
            final Optional<String> result;
            if (c == '"') {
                result = Optional.of("");
            } else {
                result = lexStringGen().map(rest -> c + rest);
            }
            return result;
        });
    }

    private Optional<Token<?>> lexInterpolatedString() {
        // Just lex the whole string for now; reenter the lexer from the parser when doing the interpolation.

        input.read();
        return lexString();
    }

    private Optional<Token<?>> lexChar() {
        input.read();
        return input.read().flatMap(c -> input
                .read()
                .filter(last -> last == '\'')
                .map(endQuote -> new Token.CharToken(c))
        );
    }

    private Optional<Token<?>> lexNumber() {
        return isLexedSignPositive()
                .flatMap(isPositive -> lexAbsoluteNumber().map(n -> isPositive ? n : n.negate()))
                .map(Token.NumberToken::new);
    }

    private Optional<Boolean> isLexedSignPositive() {
        return input.lookAhead().map(c -> {
            final boolean result;
            if (c == '-') {
                input.read();
                result = false;
            } else if (c == '+') {
                input.read();
                result = true;
            } else {
                result = true;
            }
            return result;
        });
    }

    private Optional<BigDecimal> lexAbsoluteNumber() {
        return input
                .read()
                .map(first -> new BigDecimal(first + lexAbsoluteNumberGen()));
    }

    private String lexAbsoluteNumberGen() {
        return input
                .lookAhead()
                .map(c -> {
                    final String result;
                    if (Character.isDigit(c) || c == '.') {
                        input.read();
                        result = c + lexAbsoluteNumberGen();
                    } else {
                        result = "";
                    }
                    return result;
                })
                .orElse("");
    }

    private Token<?> lexSingleLineComment() {
        input.read();
        input.read();
        return new Token.Comment(lexSingleLineCommentGen());
    }

    private String lexSingleLineCommentGen() {
        return input
                .lookAhead()
                .filter(c -> c != '\n')
                .map(c -> {
                    input.read();
                    return c + lexSingleLineCommentGen();
                })
                .orElse("");
    }

    private Optional<Token<?>> lexMultiLineComment() {
        input.read();
        input.read();
        return lexMultiLineCommentGen(0).map(Token.Comment::new);
    }

    private Optional<String> lexMultiLineCommentGen(long nestingLevel) {
        return input.lookAhead(2).flatMap(s -> {
            final Optional<String> result;
            switch (s) {
                case "/*":
                    input.read();
                    input.read();
                    result = lexMultiLineCommentGen(nestingLevel + 1)
                            .map(rest -> "/*" + rest);
                    break;

                case "*/":
                    input.read();
                    input.read();
                    if (nestingLevel <= 0) {
                        result = Optional.of("");
                    } else {
                        result = lexMultiLineCommentGen(nestingLevel - 1)
                                .map(rest -> "*/" + rest);
                    }
                    break;

                default:
                    result = input
                            .read()
                            .flatMap(c -> lexMultiLineCommentGen(nestingLevel)
                                    .map(rest -> c + rest)
                            );
                    break;
            }
            return result;
        });
    }

    private Optional<Token<?>> lexOperator() {
        return input.read().flatMap(c -> {
            final Optional<Token<?>> result;
            switch (c) {
                case ',':
                    result = Optional.of(new Token.SubItemSeparator());
                    break;
                case '-':
                    result = lexWithLeadingMinus();
                    break;
                case '<':
                    result = lexWithLeadingLeftAngleBracket();
                    break;
                case '=':
                    result = lexWithLeadingEquals();
                    break;
                case '.':
                    result = Optional.of(new Token.Dot());
                    break;
                case '!':
                    result = lexWithLeadingExclamationMark();
                    break;
                case '>':
                    result = lexWithLeadingRightAngleBracket();
                    break;
                case '~':
                    result = Optional.of(new Token.BitwiseNot());
                    break;
                case '^':
                    result = Optional.of(new Token.BitwiseXor());
                    break;
                case '|':
                    result = lexWithLeadingVerticalBar();
                    break;
                case '&':
                    result = lexWithLeadingAmpersand();
                    break;
                case '+':
                    result = Optional.of(new Token.Add());
                    break;
                case '*':
                    result = Optional.of(new Token.Multiply());
                    break;
                case '/':
                    result = Optional.of(new Token.Divide());
                    break;
                case '%':
                    result = Optional.of(new Token.Modulo());
                    break;
                case ':':
                    result = Optional.of(new Token.Colon());
                    break;
                case '{':
                    result = Optional.of(new Token.OpenBrace());
                    break;
                case '}':
                    result = Optional.of(new Token.CloseBrace());
                    break;
                case '(':
                    result = Optional.of(new Token.OpenParentheses());
                    break;
                case ')':
                    result = Optional.of(new Token.CloseParentheses());
                    break;
                case '[':
                    result = Optional.of(new Token.OpenSquareBracket());
                    break;
                case ']':
                    result = Optional.of(new Token.CloseSquareBracket());
                    break;

                default:
                    throw new LexingException(format(
                            "no operator starting with '%c' exists",
                            c
                    ));
            }
            return result;
        });
    }

    private Optional<Token<?>> lexWithLeadingMinus() {
        return input.lookAhead().map(ahead -> {
            final Token<?> subResult;
            if (ahead == '>') {
                input.read();
                subResult = new Token.LambdaArrow();
            } else {
                subResult = new Token.Subtract();
            }
            return subResult;
        });
    }

    private Optional<Token<?>> lexWithLeadingLeftAngleBracket() {
        return Optional.of(input
                .lookAhead()
                .<Token<?>>map(ahead -> {
                    final Token<?> subResult;
                    switch (ahead) {
                        case '-':
                            input.read();
                            subResult = new Token.Bind();
                            break;

                        case '<':
                            input.read();
                            subResult = new Token.ShiftLeft();
                            break;

                        case '=':
                            input.read();
                            subResult = new Token.LessThanOrEquals();
                            break;

                        default:
                            subResult = new Token.LessThan();
                            break;
                    }
                    return subResult;
                })
                .orElse(new Token.LessThan())
        );
    }

    private Optional<Token<?>> lexWithLeadingEquals() {
        return Optional.of(input
                .lookAhead()
                .filter(ahead -> ahead == '=')
                .<Token<?>>map(voidValue -> {
                    input.read();
                    return new Token.Equals();
                })
                .orElse(new Token.Assign())
        );
    }

    private Optional<Token<?>> lexWithLeadingExclamationMark() {
        return Optional.of(input
                .lookAhead()
                .filter(ahead -> ahead == '=')
                .<Token<?>>map(voidValue -> {
                    input.read();
                    return new Token.NotEquals();
                })
                .orElse(new Token.Not())
        );
    }

    private Optional<Token<?>> lexWithLeadingRightAngleBracket() {
        return Optional.of(input
                .lookAhead()
                .<Token<?>>map(ahead -> {
                    final Token<?> subResult;
                    switch (ahead) {
                        case '>':
                            input.read();
                            subResult = new Token.ShiftRight();
                            break;

                        case '=':
                            input.read();
                            subResult = new Token.GreaterThanOrEquals();
                            break;

                        default:
                            subResult = new Token.GreaterThan();
                            break;
                    }
                    return subResult;
                })
                .orElse(new Token.GreaterThan())
        );
    }

    private Optional<Token<?>> lexWithLeadingVerticalBar() {
        return Optional.of(input
                .lookAhead()
                .filter(ahead -> ahead == '|')
                .<Token<?>>map(ahead -> {
                    input.read();
                    return new Token.Or();
                })
                .orElse(new Token.BitwiseOr())
        );
    }

    private Optional<Token<?>> lexWithLeadingAmpersand() {
        return Optional.of(input
                .lookAhead()
                .filter(ahead -> ahead == '&')
                .<Token<?>>map(ahead -> {
                    input.read();
                    return new Token.And();
                })
                .orElse(new Token.BitwiseAnd())
        );
    }

    private Token<?> lexBooleanOrKeywordOrIdentifier(String word) {
        final Token<?> result;
        switch (word) {
            case "true":
                result = new Token.BooleanToken(true);
                break;

            case "false":
                result = new Token.BooleanToken(false);
                break;

            default:
                result = Keyword
                        .parse(word)
                        .<Token<?>>map(keywords::get)
                        .orElse(new Token.Identifier(word));
                break;
        }
        return result;
    }
}