package lexing;

import static java.util.Objects.requireNonNull;

/**
 * A lexing error.
 */
public final class LexingException extends RuntimeException {
    public LexingException(String message) {
        super(requireNonNull(message));
    }
}
