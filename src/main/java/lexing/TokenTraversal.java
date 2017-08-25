package lexing;

import java.util.List;

import static java.util.Collections.unmodifiableList;

public final class TokenTraversal {
    private final TokenTraverser tokenTraverser;
    private final List<Token<?>> tokens;

    public TokenTraversal(TokenTraverser traverser, List<Token<?>> tokens) {
        tokenTraverser = traverser;
        this.tokens = unmodifiableList(tokens);
    }

    public TokenTraverser getTokenTraverser() {
        return tokenTraverser;
    }

    public List<Token<?>> getTokens() {
        return tokens;
    }
}
