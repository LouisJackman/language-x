package lexing;

import junit.framework.TestCase;

public class SourcePositionTest extends TestCase {
    public void testStart() {
        SourcePosition position = SourcePosition.start();
        assertEquals(0, position.getRow());
        assertEquals(0, position.getColumn());
        assertEquals(0, position.getPosition());
        assertFalse(position.isNewLineStarted());
    }

    public void testUpdate() {
        SourcePosition position1 = SourcePosition.start();

        SourcePosition position2 = position1.update('\n');
        assertEquals(0, position2.getRow());
        assertEquals(1, position2.getColumn());
        assertEquals(1, position2.getPosition());
        assertTrue(position2.isNewLineStarted());

        SourcePosition position3 = position2.update(' ');
        assertEquals(1, position3.getRow());
        assertEquals(0, position3.getColumn());
        assertEquals(2, position3.getPosition());
        assertFalse(position3.isNewLineStarted());
    }
}
