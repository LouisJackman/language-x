package lexing;

import util.Lists;

import java.util.List;
import java.util.Optional;

import static java.util.Collections.emptyList;
import static java.util.Optional.empty;
import static util.Lists.cons;

public final class TokenTraverser {
    private final List<Token<?>> tokens;
    private final int position;

    private TokenTraverser(List<Token<?>> tokens, int position) {
        this.tokens = tokens;
        this.position = position;
    }

    public static TokenTraverser start(List<Token<?>> tokens) {
        return new TokenTraverser(tokens, 0);
    }

    public Optional<TokenTraversal> read(int n) {
        final Optional<TokenTraversal> result;
        if (n <= 0) {
            result = Optional.of(new TokenTraversal(this, emptyList()));
        } else {
            int newPosition = position + n;
            if (tokens.size() <= newPosition) {
                result = empty();
            } else {
                result = Optional.of(new TokenTraversal(
                        new TokenTraverser(
                                tokens,
                                newPosition
                        ),
                        tokens.subList(position, newPosition)
                ));
            }
        }
        return result;
    }

    public Optional<TokenTraversal> read() {
        return read(1);
    }

    private Optional<List<Token<?>>> lookAheadGen(int n, int m) {
        final Optional<List<Token<?>>> result;
        if (m <= n) {
            result = Optional.of(emptyList());
        } else {
            int newPosition = position + n;
            if (tokens.size() <= newPosition) {
                result = empty();
            } else {
                result = lookAheadGen(n + 1, m).map(rest -> cons(tokens.get(newPosition), rest));
            }
        }
        return result;
    }

    public Optional<List<Token<?>>> lookAhead(int n) {
        return lookAheadGen(0, n);
    }

    public Optional<Token<?>> lookAhead() {
        return lookAhead(1).flatMap(Lists::head);
    }
}
