package parsing;

import static java.util.Objects.requireNonNull;

/**
 * A lexing error.
 */
public class LexingException extends RuntimeException {
    public LexingException(String message) {
        super(requireNonNull(message));
    }
}
