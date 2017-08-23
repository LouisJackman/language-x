package lexing;

import java.util.Map;
import java.util.Optional;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import static java.util.Collections.unmodifiableMap;

/**
 * The language keywords.
 */
public enum Keyword {
    ABSTRACT("abstract"),
    ACTOR("actor"),
    CASE("case"),
    CLASS("class"),
    CONTINUE("continue"),
    DEFAULT("default"),
    ELSE("else"),
    EXTENDS("extends"),
    FOR("for"),
    GET("get"),
    IF("if"),
    IMPLEMENTS("implements"),
    IMPORT("import"),
    INTERFACE("interface"),
    INTERNAL("internal"),
    OVERRIDE("override"),
    PACKAGE("package"),
    PUBLIC("public"),
    RECEIVE("receive"),
    SUPER("super"),
    SWITCH("switch"),
    THROW("throw"),
    TIMEOUT("timeout"),
    UNDERSCORE("_"),
    V("v"),
    VAR("var");

    private static Map<String, Keyword> parseTable = unmodifiableMap(Stream
            .of(Keyword.values())
            .collect(Collectors.toMap(Keyword::toString, Function.identity()))
    );

    private final String string;

    Keyword(String s) {
        string = s;
    }

    /**
     * Gets the keyword from its string representation.
     */
    public static Optional<Keyword> parse(String s) {
        return parseTable.containsKey(s)
                ? Optional.of(parseTable.get(s))
                : Optional.empty();
    }

    @Override
    public String toString() {
        return string;
    }
}
