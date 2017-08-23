package util;

/**
 * Utilities for exceptions.
 */
public final class Exceptions {
    private Exceptions() {
        throw new UtilityClassInstantiatedException();
    }

    /**
     * Catch compile-check exceptions in {@code f} and rethrow as runtime exceptions.
     */
    public static <Result> Result unchecked(Checked<Result> f) {
        try {
            return f.run();
        } catch (Exception e) {
            throw new UncheckedException(e);
        }
    }

    public interface Checked<Result> {
        Result run() throws Exception;
    }
}
