package util;

public final class ExhaustedConditionException extends RuntimeException {
    public ExhaustedConditionException() {
    }

    public ExhaustedConditionException(final String message) {
        super(message);
    }
}
