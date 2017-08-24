package lexing;

import java.math.BigDecimal;
import java.util.Optional;

import static java.util.Objects.requireNonNull;

/**
 * All lexable tokens.
 */
public abstract class Token<T> {
    public Optional<T> getValue() {
        return Optional.empty();
    }

    @Override
    public String toString() {
        String typeName = this.getClass().getSimpleName();
        return getValue()
                .map(x -> typeName + ": " + x)
                .orElse(typeName);
    }

    /**
     * A token that just represents itself.
     */
    public abstract static class Symbolic extends Token<Void> {
    }

    /**
     * A token that carries an associated value.
     */
    public abstract static class Valued<Value> extends Token<Value> {
        private final Value value;

        Valued(Value value) {
            this.value = requireNonNull(value);
        }

        public Optional<Value> getValue() {
            return Optional.of(value);
        }
    }

    public static final class BooleanToken extends Valued<Boolean> {
        public BooleanToken(boolean b) {
            super(b);
        }
    }

    public static final class CharToken extends Valued<Character> {
        public CharToken(Character c) {
            super(c);
        }
    }

    /**
     * Comments are currently lexed, but they could also be skipped if we know their values are being used for, say,
     * document generation.
     */
    public static final class Comment extends Valued<String> {
        public Comment(String s) {
            super(s);
        }
    }

    public static final class Identifier extends Valued<String> {
        public Identifier(String s) {
            super(s);
        }
    }

    public static final class InterpolatedString extends Valued<String> {
        public InterpolatedString(String v) {
            super(v);
        }
    }

    public static final class NumberToken extends Valued<BigDecimal> {
        public NumberToken(BigDecimal n) {
            super(n);
        }
    }

    public static final class StringToken extends Valued<String> {
        public StringToken(String s) {
            super(s);
        }
    }

    public static final class Version extends Valued<BigDecimal> {
        public Version(BigDecimal n) {
            super(n);
        }
    }

    public static final class Abstract extends Symbolic {
    }

    public static final class Add extends Symbolic {
    }

    public static final class And extends Symbolic {
    }

    public static final class Assign extends Symbolic {
    }

    public static final class Bind extends Symbolic {
    }

    public static final class BitwiseAnd extends Symbolic {
    }

    public static final class BitwiseNot extends Symbolic {
    }

    public static final class BitwiseOr extends Symbolic {
    }

    public static final class BitwiseXor extends Symbolic {
    }

    public static final class Case extends Symbolic {
    }

    public static final class Class extends Symbolic {
    }

    public static final class CloseBrace extends Symbolic {
    }

    public static final class CloseParentheses extends Symbolic {
    }

    public static final class CloseSquareBracket extends Symbolic {
    }

    public static final class Colon extends Symbolic {
    }

    public static final class Continue extends Symbolic {
    }

    public static final class Default extends Symbolic {
    }

    public static final class Divide extends Symbolic {
    }

    public static final class Do extends Symbolic {
    }

    public static final class Else extends Symbolic {
    }

    public static final class Equals extends Symbolic {
    }

    public static final class Extends extends Symbolic {
    }

    public static final class For extends Symbolic {
    }

    public static final class Get extends Symbolic {
    }

    public static final class GreaterThan extends Symbolic {
    }

    public static final class GreaterThanOrEquals extends Symbolic {
    }

    public static final class If extends Symbolic {
    }

    public static final class Ignore extends Symbolic {
    }

    public static final class Implements extends Symbolic {
    }

    public static final class Import extends Symbolic {
    }

    public static final class Interface extends Symbolic {
    }

    public static final class Internal extends Symbolic {
    }

    public static final class LambdaArrow extends Symbolic {
    }

    public static final class LessThan extends Symbolic {
    }

    public static final class LessThanOrEquals extends Symbolic {
    }

    public static final class Dot extends Symbolic {
    }

    public static final class Modulo extends Symbolic {
    }

    public static final class Multiply extends Symbolic {
    }

    public static final class Not extends Symbolic {
    }

    public static final class NotEquals extends Symbolic {
    }

    public static final class OpenBrace extends Symbolic {
    }

    public static final class OpenParentheses extends Symbolic {
    }

    public static final class OpenSquareBracket extends Symbolic {
    }

    public static final class Or extends Symbolic {
    }

    public static final class OverrideToken extends Symbolic {
    }

    public static final class Package extends Symbolic {
    }

    public static final class Public extends Symbolic {
    }

    public static final class Select extends Symbolic {
    }

    public static final class ShiftLeft extends Symbolic {
    }

    public static final class ShiftRight extends Symbolic {
    }

    public static final class SubItemSeparator extends Symbolic {
    }

    public static final class Subtract extends Symbolic {
    }

    public static final class Super extends Symbolic {
    }

    public static final class Switch extends Symbolic {
    }

    public static final class Throw extends Symbolic {
    }

    public static final class Timeout extends Symbolic {
    }

    public static final class Var extends Symbolic {
    }
}
