package parsing;

import junit.framework.TestCase;
import lexing.Token;
import lexing.TokenTraversal;
import lexing.TokenTraverser;

import java.math.BigDecimal;
import java.util.Optional;

import static java.util.Arrays.asList;

public final class TokenTraverserTest extends TestCase {

    public void test() {
        TokenTraverser parser = TokenTraverser.start(asList(
                new Token.Select(),
                new Token.Class(),
                new Token.StringToken("abc"),
                new Token.NumberToken(new BigDecimal("12.34"))
        ));

        assertEquals(
                Optional.of(asList(new Token.Select(), new Token.Class())),
                parser.lookAhead(2)
        );

        Optional<TokenTraversal> optionalRead = parser.read(3);
        assertTrue(optionalRead.isPresent());
        TokenTraversal read = optionalRead.get();
        assertEquals(
                asList(
                        new Token.Select(),
                        new Token.Class(),
                        new Token.StringToken("abc")
                ),
                read.getTokens()
        );

        assertEquals(
                Optional.of(new Token.NumberToken(new BigDecimal("12.34"))),
                read.getTokenTraverser().lookAhead()
        );
    }
}
