package lexing;

/**
 * A position in a source file which is aware of newlines.
 */
public final class SourcePosition {
    private final boolean newLineStarted;
    private final long row;
    private final long column;
    private final long position;

    public SourcePosition(boolean newLineStarted, long row, long column, long position) {
        this.newLineStarted = newLineStarted;
        this.row = row;
        this.column = column;
        this.position = position;
    }

    /**
     * Start a new position at the starting row, column, and position.
     */
    public static SourcePosition start() {
        return new SourcePosition(false, 0, 0, 0);
    }

    public boolean isNewLineStarted() {
        return newLineStarted;
    }

    public long getRow() {
        return row;
    }

    public long getColumn() {
        return column;
    }

    public long getPosition() {
        return position;
    }

    /**
     * Move the position by one according to {@code nextChar}. If it is a newline, the next update will reset the column
     * and increase the row.
     */
    public SourcePosition update(char nextChar) {
        final long row;
        final long column;
        if (isNewLineStarted()) {
            row = getRow() + 1;
            column = 0;
        } else {
            row = getRow();
            column = getColumn() + 1;
        }

        boolean newLineStarted = ((nextChar == '\n') || (nextChar == '\r'));
        long position = getPosition() + 1;
        return new SourcePosition(newLineStarted, row, column, position);
    }
}
