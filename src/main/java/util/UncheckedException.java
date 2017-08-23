package util;

public final class UncheckedException extends RuntimeException {
    public UncheckedException(Exception e) {
        super(e);
    }
}
