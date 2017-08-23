package lexing;

import junit.framework.TestCase;

import java.io.ByteArrayInputStream;
import java.math.BigDecimal;
import java.nio.charset.StandardCharsets;
import java.util.Optional;

public class LexerTest extends TestCase {

    private static Lexer createLexer(String testInput) {
        byte[] bytes = testInput.getBytes(StandardCharsets.UTF_8);
        SourceFileStream stream = new SourceFileStream(new ByteArrayInputStream(bytes));
        return new Lexer(stream);
    }

    public void testEmpty() {
        assertFalse(createLexer("    \t  \n      ").hasNext());
    }

    public void testIdentifier() {
        Lexer lexer = createLexer("    \t  \n      abc");
        Token<?> token = lexer.next();
        assertTrue(token instanceof lexing.Token.Identifier);
        Optional<?> value = token.getValue();
        assertEquals("abc", ((String) value.get()));
    }

    public void testKeywords() {
        Lexer lexer = createLexer("    class\t  \n  public    abc");
        Token<?> token1 = lexer.next();
        Token<?> token2 = lexer.next();
        Token<?> token3 = lexer.next();
        assertTrue(token1 instanceof lexing.Token.Class);
        assertTrue(token2 instanceof lexing.Token.Public);
        assertTrue(token3 instanceof lexing.Token.Identifier);
        assertEquals("abc", token3.getValue().get());
    }

    public void testNumbers() {
        Lexer lexer = createLexer("    23  \t     \t\t\n   23   +32 0.32    \t123123123.32");
        assertEquals(new BigDecimal(23), lexer.next().getValue().get());
        assertEquals(new BigDecimal(23), lexer.next().getValue().get());
        assertEquals(new BigDecimal(32), lexer.next().getValue().get());
        assertEquals(new BigDecimal("0.32"), lexer.next().getValue().get());
        assertEquals(new BigDecimal("123123123.32"), lexer.next().getValue().get());
    }

    public void testChars() {
        Lexer lexer = createLexer("  'a'   \t \n\n\n 'd'    '/'");
        assertEquals('a', lexer.next().getValue().get());
        assertEquals('d', lexer.next().getValue().get());
        assertEquals('/', lexer.next().getValue().get());
    }

    public void testStrings() {
        Lexer lexer = createLexer("  \"abcdef\"   \t \n\n\n\"'123'\"");
        assertEquals("abcdef", lexer.next().getValue().get());
        assertEquals("'123'", lexer.next().getValue().get());
    }

    public void testInterpolatedStrings() {
        // TODO: test actual interpolation once the parser is complete.
    }

    public void testOperators() {
        Lexer lexer = createLexer("   <= \t  \n ~ ! ^   >> != ");
        assertTrue(lexer.next() instanceof Token.LessThanOrEquals);
        assertTrue(lexer.next() instanceof Token.BitwiseNot);
        assertTrue(lexer.next() instanceof Token.Not);
        assertTrue(lexer.next() instanceof Token.BitwiseXor);
        assertTrue(lexer.next() instanceof Token.ShiftRight);
        assertTrue(lexer.next() instanceof Token.NotEquals);
    }

    public void testSingleLineComments() {
        Lexer lexer = createLexer("      //    //  abc   ");
        Token<?> token = lexer.next();
        assertTrue(token instanceof Token.Comment);
        assertTrue(((String) token.getValue().get()).contains("abc"));
    }

    public void testMultiLineComments() {
        Lexer lexer = createLexer("  /*   /* 123 */      */ ");
        Token<?> token = lexer.next();
        assertTrue(token instanceof Token.Comment);
        assertTrue(((String) token.getValue().get()).contains("/* 123 */"));
    }

    public void testBooleans() {
        Lexer lexer = createLexer("  true false   \n\t   /*   */ false true");
        Token<?> token1 = lexer.next();
        Token<?> token2 = lexer.next();
        lexer.next();
        Token<?> token3 = lexer.next();
        Token<?> token4 = lexer.next();
        assertTrue(token1 instanceof Token.BooleanToken);
        assertTrue(token2 instanceof Token.BooleanToken);
        assertTrue(token3 instanceof Token.BooleanToken);
        assertTrue(token4 instanceof Token.BooleanToken);
        assertEquals(true, token1.getValue().get());
        assertEquals(false, token2.getValue().get());
        assertEquals(false, token3.getValue().get());
        assertEquals(true, token4.getValue().get());
    }

    public void testVersion() {
        Lexer lexer = createLexer("v10.23");
        Token<?> token = lexer.next();
        assertEquals(new BigDecimal("10.23"), token.getValue().get());
    }
}
