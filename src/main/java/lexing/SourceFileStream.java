package lexing;

import java.io.InputStream;
import java.util.LinkedList;
import java.util.List;
import java.util.Optional;
import java.util.function.Function;

import static java.util.Objects.requireNonNull;
import static java.util.Optional.empty;
import static util.Exceptions.unchecked;

/**
 * A stream of a file that supports arbitrary lookaheads and tracks the current position.
 */
public final class SourceFileStream {
    private final InputStream input;
    private final List<Character> lookAhead = new LinkedList<>();
    private boolean empty = false;
    private SourcePosition position = SourcePosition.start();

    /**
     * Wraps an {@link java.io.InputStream} in a {@link SourceFileStream}.
     */
    public SourceFileStream(InputStream input) {
        this.input = requireNonNull(input);
    }

    /**
     * Is the underlying stream finished?
     */
    public boolean isEmpty() {
        return empty;
    }

    public SourcePosition getPosition() {
        return position;
    }

    /**
     * Read {@code n} characters ahead, updating the position.
     *
     * @return The read characters.
     */
    public Optional<String> read(int n) {
        final Optional<String> result;
        if (n <= 0) {
            result = Optional.of("");
        } else if (lookAhead.isEmpty()) {
            result = maybeReadChar(Function.identity()).flatMap(c -> {
                position = getPosition().update(c);
                return read(n - 1).map(next -> c + next);
            });
        } else {
            char next = lookAhead.remove(0);
            position = getPosition().update(next);
            result = read(n - 1).map(rest -> String.valueOf(next) + rest);
        }
        return result;
    }

    /**
     * @see #read(int)
     */
    public Optional<Character> read() {
        return read(1).map(s -> s.charAt(0));
    }

    /**
     * Look {@code n} characters ahead, not updating the position.
     *
     * @return The characters ahead.
     */
    public Optional<String> lookAhead(int n) {
        return lookAheadGen(1, n);
    }

    /**
     * @see #lookAhead(int)
     */
    public Optional<Character> lookAhead() {
        return lookAhead(1).map(s -> s.charAt(0));
    }

    private Optional<String> lookAheadGen(int i, int m) {
        final Optional<String> result;
        if (m < i) {
            result = Optional.of("");
        } else if (lookAhead.size() < i) {
            result = maybeReadChar(lookAhead::add).flatMap(_x -> lookAheadGen(i, m));
        } else {
            String next = String.valueOf(lookAhead.get(i - 1));
            result = lookAheadGen(i + 1, m).map(x -> next + x);
        }
        return result;
    }

    private <T> Optional<T> maybeReadChar(Function<Character, T> f) {
        final Optional<T> result;
        int next = unchecked(input::read);
        if (next == -1) {
            empty = true;
            result = empty();
        } else {
            result = Optional.of(f.apply((char) next));
        }
        return result;
    }

}
